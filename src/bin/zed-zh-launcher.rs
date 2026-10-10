#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
mod desktop {
    use anyhow::{Context, Result};
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use serde_json::{Value, json};
    use std::{
        collections::HashSet,
        fs,
        os::windows::process::CommandExt,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        sync::mpsc,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::{
            ProcessStatus::K32EnumProcesses,
            Threading::{
                OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
            },
        },
    };
    const HIDDEN: u32 = 0x0800_0000;

    fn root() -> Result<PathBuf> {
        let exe = std::env::current_exe()?;
        exe.ancestors()
            .find(|path| path.join("runtime/supervisor.py").is_file())
            .map(Path::to_owned)
            .context("Cannot locate local runtime")
    }
    fn spawn_python(root: &Path, script: &str, args: &[String]) -> Result<()> {
        Command::new(root.join(".venv/Scripts/python.exe"))
            .arg(root.join(script))
            .args(args)
            .current_dir(root)
            .env_remove("ZED_I18N_API_KEY")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(HIDDEN)
            .spawn()?;
        Ok(())
    }
    fn settings(root: &Path) -> Value {
        fs::read(root.join("local/settings.json"))
            .ok()
            .and_then(|bytes| {
                serde_json::from_slice(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes))
                    .ok()
            })
            .unwrap_or_else(|| json!({"zed_exe":"D:\\Zed\\Zed.exe","watch_enabled":true}))
    }
    fn repair(root: &Path) -> Result<()> {
        let result = Command::new("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(root.join("runtime/install-startup.ps1"))
            .arg("-RepairOnly")
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(HIDDEN)
            .status()?;
        if !result.success() {
            anyhow::bail!("Shortcut repair failed");
        }
        Ok(())
    }
    fn zed_pids(exe: &str) -> Vec<u32> {
        let mut pids = vec![0u32; 8192];
        let mut bytes = 0;
        if unsafe { K32EnumProcesses(pids.as_mut_ptr(), (pids.len() * 4) as u32, &mut bytes) } == 0
        {
            return vec![];
        }
        pids.truncate(bytes as usize / 4);
        pids.into_iter()
            .filter(|pid| {
                let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, *pid) };
                if handle.is_null() {
                    return false;
                }
                let mut text = vec![0u16; 32768];
                let mut length = text.len() as u32;
                let ok = unsafe {
                    QueryFullProcessImageNameW(handle, 0, text.as_mut_ptr(), &mut length)
                };
                unsafe {
                    CloseHandle(handle);
                }
                ok != 0
                    && String::from_utf16_lossy(&text[..length as usize]).eq_ignore_ascii_case(exe)
            })
            .collect()
    }
    fn watch(root: &Path) -> Result<()> {
        use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
        use windows_sys::Win32::System::Threading::CreateMutexW;
        let name: Vec<u16> = "Local\\ZedZhShortcutGuard"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mutex = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if mutex.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe {
                CloseHandle(mutex);
            }
            return Ok(());
        }
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |event| {
                let _ = tx.send(event);
            },
            notify::Config::default(),
        )?;
        let current = settings(root);
        let exe = current["zed_exe"].as_str().unwrap_or("D:\\Zed\\Zed.exe");
        let mut roots = vec![
            Path::new(exe)
                .parent()
                .context("No Zed install directory")?
                .to_owned(),
        ];
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            roots.push(PathBuf::from(profile).join("Desktop"));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let appdata = PathBuf::from(appdata);
            roots.push(appdata.join("Microsoft/Windows/Start Menu/Programs/Zed"));
            roots
                .push(appdata.join("Microsoft/Internet Explorer/Quick Launch/User Pinned/TaskBar"));
        }
        for directory in roots {
            if directory.is_dir() {
                watcher.watch(&directory, RecursiveMode::Recursive)?;
            }
        }
        repair(root)?;
        let mut seen: HashSet<_> = zed_pids(exe).into_iter().collect();
        let mut dirty = false;
        let mut last_repair = Instant::now();
        let mut last_scan = Instant::now();
        loop {
            if rx.recv_timeout(Duration::from_secs(2)).is_ok() {
                dirty = true;
            }
            while rx.try_recv().is_ok() {
                dirty = true;
            }
            if dirty && last_repair.elapsed() > Duration::from_secs(5) {
                repair(root)?;
                let _ = spawn_python(root, "runtime/update_compat.py", &[]);
                dirty = false;
                last_repair = Instant::now();
            }
            if last_scan.elapsed() < Duration::from_secs(5) {
                continue;
            }
            last_scan = Instant::now();
            let config = settings(root);
            let exe = config["zed_exe"].as_str().unwrap_or("D:\\Zed\\Zed.exe");
            let pids = zed_pids(exe);
            let next: HashSet<_> = pids.iter().copied().collect();
            if config["watch_enabled"].as_bool() != Some(false) {
                for pid in next.difference(&seen) {
                    let _ = spawn_python(
                        root,
                        "runtime/supervisor.py",
                        &["--attach".into(), pid.to_string()],
                    );
                }
            }
            seen = next;
        }
    }
    pub fn run() -> Result<()> {
        let root = root()?;
        let args: Vec<String> = std::env::args().skip(1).collect();
        if args.first().is_some_and(|value| value == "--watch") {
            return watch(&root);
        }
        if args.first().is_some_and(|value| value == "--settings") {
            return spawn_python(&root, "runtime/ai_settings.py", &[]);
        }
        let _ = Command::new(std::env::current_exe()?)
            .arg("--watch")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(HIDDEN)
            .spawn();
        spawn_python(&root, "runtime/supervisor.py", &args)
    }
}

fn main() {
    #[cfg(windows)]
    if let Err(error) = desktop::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("The desktop launcher requires Windows");
        std::process::exit(1);
    }
}
