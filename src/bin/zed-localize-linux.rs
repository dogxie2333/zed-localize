#[cfg(target_os = "linux")]
mod linux {
    use anyhow::{Context, Result, bail, ensure};
    use clap::{Parser, Subcommand};
    use serde_json::json;
    use std::{
        fs,
        os::unix::process::CommandExt,
        path::{Path, PathBuf},
        process::{Child, Command},
        time::Instant,
    };
    use zed_zh_addon::catalog::{Plan, hash_file, make_plan_for_language};
    use zed_zh_addon::linux_display::{self, DisplayServer};

    #[derive(Parser)]
    #[command(
        version,
        about = "Ubuntu x86_64 localization companion for official Zed"
    )]
    pub struct Args {
        #[arg(long)]
        zed: Option<PathBuf>,
        #[arg(long, default_value = "zh-CN")]
        language: String,
        #[arg(long, value_enum, default_value_t)]
        display_server: DisplayServer,
        #[arg(long)]
        resources: Option<PathBuf>,
        #[command(subcommand)]
        command: Action,
    }
    #[derive(Subcommand)]
    enum Action {
        Plan,
        Launch {
            #[arg(last = true)]
            arguments: Vec<String>,
        },
        TestMemory,
    }
    struct Traced {
        child: Child,
        armed: bool,
    }
    impl Drop for Traced {
        fn drop(&mut self) {
            if self.armed {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
        }
    }
    fn trace(command: &mut Command) -> Result<Traced> {
        unsafe {
            command.pre_exec(|| {
                if libc::ptrace(
                    libc::PTRACE_TRACEME,
                    0,
                    std::ptr::null_mut::<libc::c_void>(),
                    std::ptr::null_mut::<libc::c_void>(),
                ) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn()?;
        let traced = Traced { child, armed: true };
        let mut status = 0;
        ensure!(
            unsafe { libc::waitpid(traced.child.id() as i32, &mut status, 0) } > 0,
            "Cannot wait for owned child"
        );
        ensure!(
            libc::WIFSTOPPED(status) && libc::WSTOPSIG(status) == libc::SIGTRAP,
            "Owned child did not stop at exec"
        );
        ensure!(
            unsafe {
                libc::ptrace(
                    libc::PTRACE_SETOPTIONS,
                    traced.child.id() as i32,
                    0,
                    libc::PTRACE_O_EXITKILL,
                )
            } >= 0,
            "Cannot protect traced child"
        );
        Ok(traced)
    }
    fn word(pid: u32, address: usize) -> Result<libc::c_long> {
        unsafe {
            *libc::__errno_location() = 0;
        }
        let value = unsafe {
            libc::ptrace(
                libc::PTRACE_PEEKDATA,
                pid as i32,
                address as *mut libc::c_void,
                0,
            )
        };
        ensure!(
            value != -1 || unsafe { *libc::__errno_location() } == 0,
            "Cannot read owned child memory"
        );
        Ok(value)
    }
    fn read(pid: u32, address: usize, length: usize) -> Result<Vec<u8>> {
        let unit = std::mem::size_of::<libc::c_long>();
        let mut data = Vec::with_capacity(length + unit);
        for offset in (0..length).step_by(unit) {
            data.extend_from_slice(&word(pid, address + offset)?.to_ne_bytes());
        }
        data.truncate(length);
        Ok(data)
    }
    fn write(pid: u32, address: usize, bytes: &[u8]) -> Result<()> {
        let unit = std::mem::size_of::<libc::c_long>();
        for offset in (0..bytes.len()).step_by(unit) {
            let mut value = word(pid, address + offset)?.to_ne_bytes();
            let count = unit.min(bytes.len() - offset);
            value[..count].copy_from_slice(&bytes[offset..offset + count]);
            ensure!(
                unsafe {
                    libc::ptrace(
                        libc::PTRACE_POKEDATA,
                        pid as i32,
                        (address + offset) as *mut libc::c_void,
                        libc::c_long::from_ne_bytes(value),
                    )
                } >= 0,
                "Cannot write owned child memory"
            );
        }
        Ok(())
    }
    fn base(pid: u32, exe: &Path) -> Result<usize> {
        ensure!(
            fs::read_link(format!("/proc/{pid}/exe"))?.canonicalize()? == exe.canonicalize()?,
            "Owned process executable changed"
        );
        for line in fs::read_to_string(format!("/proc/{pid}/maps"))?.lines() {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() >= 6
                && fields[2] == "00000000"
                && fields[5] == exe.to_str().unwrap_or("")
            {
                return Ok(usize::from_str_radix(
                    fields[0].split('-').next().context("Invalid map")?,
                    16,
                )?);
            }
        }
        bail!("Cannot locate owned ELF image")
    }
    fn verify(pid: u32, base: usize, plan: &Plan, patched: bool) -> Result<()> {
        for entry in &plan.entries {
            let expected = hex::decode(if patched { &entry.after } else { &entry.before })?;
            ensure!(
                read(pid, base + entry.rva, expected.len())? == expected,
                "Memory verification failed"
            );
        }
        Ok(())
    }
    fn apply(pid: u32, base: usize, plan: &Plan, fail_after: Option<usize>) -> Result<()> {
        verify(pid, base, plan, false)?;
        let applied = (|| -> Result<()> {
            for (index, entry) in plan.entries.iter().enumerate() {
                write(pid, base + entry.rva, &hex::decode(&entry.after)?)?;
                if fail_after == Some(index + 1) {
                    bail!("Injected rollback test failure");
                }
            }
            verify(pid, base, plan, true)
        })();
        if applied.is_err() {
            for entry in &plan.entries {
                write(pid, base + entry.rva, &hex::decode(&entry.before)?)?;
            }
            verify(pid, base, plan, false)?;
        }
        applied
    }
    fn discover() -> Option<PathBuf> {
        let mut roots = Vec::new();
        if let Some(home) = std::env::var_os("HOME") {
            roots.push(PathBuf::from(home).join(".local/zed.app/libexec/zed-editor"));
        }
        roots.extend([
            PathBuf::from("/opt/zed.app/libexec/zed-editor"),
            PathBuf::from("/usr/lib/zed/zed-editor"),
        ]);
        roots.into_iter().find(|path| path.is_file())
    }
    pub fn run() -> Result<()> {
        let args = Args::parse();
        ensure!(
            zed_zh_addon::localization::supported(&args.language),
            "Unsupported language"
        );
        let root = args.resources.unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_owned()
        });
        let exe = args
            .zed
            .or_else(discover)
            .context("Use --zed with the official zed-editor executable")?
            .canonicalize()?;
        if let Action::Launch { arguments } = &args.command
            && args.language == "en"
        {
            let mut command = Command::new(exe);
            command.args(arguments);
            linux_display::prepare(&mut command, args.display_server)?;
            command.spawn()?;
            return Ok(());
        }
        let plan = make_plan_for_language(&exe, true, Some((&root, &args.language)))?;
        ensure!(
            plan.backend == "experimental-linux-equal-byte-rodata",
            "Expected Linux ELF executable"
        );
        if matches!(args.command, Action::Plan) {
            println!("{}", serde_json::to_string_pretty(&plan)?);
            return Ok(());
        }
        if !plan.supported {
            if let Action::Launch { arguments } = &args.command {
                eprintln!("Unverified official version; opening original UI");
                let mut command = Command::new(exe);
                command.args(arguments);
                linux_display::prepare(&mut command, args.display_server)?;
                command.spawn()?;
                return Ok(());
            }
            bail!("Unverified Linux binary; no memory changes allowed");
        }
        ensure!(!plan.entries.is_empty(), "No safe translation candidates");
        let start = Instant::now();
        let mut command = Command::new(&exe);
        let mut display_server = None;
        if let Action::Launch { arguments } = &args.command {
            display_server = Some(linux_display::prepare(&mut command, args.display_server)?);
            command.args(arguments);
        }
        if matches!(args.command, Action::TestMemory) {
            command.arg("--version");
        }
        let mut owned = trace(&mut command)?;
        let pid = owned.child.id();
        let address = base(pid, &exe)?;
        ensure!(
            hash_file(&exe)? == plan.sha256,
            "Official binary changed before apply"
        );
        if matches!(args.command, Action::TestMemory) {
            let mut mismatched = plan.clone();
            mismatched.entries[0].before = "00".repeat(plan.entries[0].source.len());
            ensure!(
                apply(pid, address, &mismatched, None).is_err(),
                "Wrong original was not rejected"
            );
            verify(pid, address, &plan, false)?;
            ensure!(
                apply(pid, address, &plan, Some(1)).is_err(),
                "Rollback fault was not injected"
            );
            verify(pid, address, &plan, false)?;
            apply(pid, address, &plan, None)?;
            for entry in &plan.entries {
                write(pid, address + entry.rva, &hex::decode(&entry.before)?)?;
            }
            verify(pid, address, &plan, false)?;
            ensure!(hash_file(&exe)? == plan.sha256, "Disk binary changed");
            println!(
                "{}",
                json!({"passed":true,"candidate_count":plan.entries.len(),"language":args.language,"wrong_original_rejected":true,"rollback":true,"restore":true,"owned_process":true,"disk_unchanged":true,"seconds":start.elapsed().as_secs_f64()})
            );
            return Ok(());
        }
        apply(pid, address, &plan, None)?;
        ensure!(
            unsafe { libc::ptrace(libc::PTRACE_DETACH, pid as i32, 0, 0) } >= 0,
            "Cannot resume owned editor"
        );
        owned.armed = false;
        println!(
            "{}",
            json!({"pid":pid,"language":args.language,"applied":plan.entries.len(),"display_server":display_server,"seconds":start.elapsed().as_secs_f64()})
        );
        Ok(())
    }
}
fn main() {
    #[cfg(target_os = "linux")]
    if let Err(error) = linux::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
    #[cfg(not(target_os = "linux"))]
    eprintln!("This executable is for Ubuntu x86_64.");
}
