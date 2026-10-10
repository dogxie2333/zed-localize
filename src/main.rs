use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, time::Instant};
#[cfg(windows)]
use std::{
    path::Path,
    process::{Command as ProcessCommand, Stdio},
};
#[cfg(windows)]
use zed_zh_addon::catalog::hash_file;
use zed_zh_addon::catalog::make_plan_with_scope;

#[derive(Parser)]
#[command(
    version,
    about = "Official Zed localization companion / 官方 Zed 局部汉化附件"
)]
struct Cli {
    /// Expand to plain settings/notifications; still experimental.
    #[arg(long, global = true)]
    extended: bool,
    #[arg(long, global = true)]
    exe: Option<PathBuf>,
    #[arg(long, global = true)]
    output: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Plan,
    TestMemory,
    Launch {
        #[arg(long)]
        experimental: bool,
        #[arg(long)]
        isolated: bool,
        #[arg(last = true)]
        arguments: Vec<String>,
    },
    Restore {
        #[arg(long)]
        pid: u32,
    },
    #[cfg(feature = "ai")]
    Translate {
        #[arg(long)]
        jobs: PathBuf,
        #[arg(long)]
        config: PathBuf,
        #[arg(long, default_value = ".cache/translations.redb")]
        cache: PathBuf,
        #[arg(long)]
        toolkit: PathBuf,
        #[arg(long, default_value = "python")]
        python: PathBuf,
        #[arg(long, default_value = "tools/community_bridge.py")]
        bridge: PathBuf,
        #[arg(long)]
        refresh: bool,
    },
    #[cfg(feature = "ai")]
    CheckUpdate,
}

fn discover_exe(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(exe) = explicit {
        ensure!(exe.is_file(), "Zed executable does not exist");
        return Ok(exe);
    }
    let mut candidates = Vec::new();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local).join("Programs/Zed/Zed.exe"));
    }
    if let Some(programs) = std::env::var_os("ProgramFiles") {
        candidates.push(PathBuf::from(programs).join("Zed/Zed.exe"));
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            if path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("bin"))
                && let Some(parent) = path.parent()
            {
                candidates.push(parent.join("Zed.exe"));
            }
            candidates.push(path.join("Zed.exe"));
        }
    }
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .context("Cannot locate Zed; provide --exe")
}

#[cfg(windows)]
fn profile_arguments(isolated: bool, test: bool) -> Result<Vec<String>> {
    if !isolated && !test {
        return Ok(vec![]);
    }
    let root = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("zed-zh-addon/profiles");
    let profile = root.join(if test { "memory-test" } else { "preview" });
    let settings = profile.join("config/settings.json");
    fs::create_dir_all(settings.parent().context("No settings directory")?)?;
    if !settings.exists() {
        fs::write(
            &settings,
            "{\"auto_update\":false,\"telemetry\":{\"metrics\":false,\"diagnostics\":false}}\n",
        )?;
    }
    Ok(vec![
        "--user-data-dir".into(),
        profile.display().to_string(),
    ])
}

#[cfg(windows)]
fn english(exe: &Path, arguments: &[String], reason: &str) -> Result<Value> {
    let child = ProcessCommand::new(exe)
        .args(arguments)
        .current_dir(exe.parent().context("No parent directory")?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(
        json!({"mode":"english-fallback", "pid":child.id(), "reason":reason, "resident_launcher":false}),
    )
}

#[cfg(windows)]
fn memory_test(exe: &Path, extended: bool) -> Result<Value> {
    use zed_zh_addon::windows_runtime::{Process, apply, restore, verify};
    let started = Instant::now();
    let plan = make_plan_with_scope(exe, extended)?;
    ensure!(plan.supported, "Executable has not been approved");
    let process = Process::spawn_suspended(exe, &profile_arguments(true, true)?)?;
    let base = process.image_base()?;
    let mut invalid = plan.clone();
    invalid.entries[0].before = "00".repeat(hex::decode(&invalid.entries[0].before)?.len());
    ensure!(
        apply(&process, base, &invalid, None).is_err(),
        "Mismatched original was accepted"
    );
    verify(&process, base, &plan.entries, false)?;
    ensure!(
        apply(&process, base, &plan, Some(1)).is_err(),
        "Injected failure did not abort"
    );
    verify(&process, base, &plan.entries, false)?;
    let count = apply(&process, base, &plan, None)?;
    verify(&process, base, &plan.entries, true)?;
    let restored = restore(&process, base, &plan)?;
    let unchanged = hash_file(exe)? == plan.sha256;
    ensure!(unchanged, "Official executable changed during test");
    Ok(
        json!({"mode":"memory-test", "pid":process.pid, "count":count, "restored":restored,
        "original_mismatch_rejected":true,"injected_failure_rolled_back":true,
        "disk_unchanged":unchanged,"extended":extended,"isolated":true,"elapsed_seconds":started.elapsed().as_secs_f64()}),
    )
}

#[cfg(windows)]
fn launch(
    exe: &Path,
    experimental: bool,
    isolated: bool,
    extra: Vec<String>,
    extended: bool,
) -> Result<Value> {
    use zed_zh_addon::windows_runtime::{Process, apply};
    let mut arguments = profile_arguments(isolated, false)?;
    arguments.extend(extra);
    if !experimental {
        return english(exe, &arguments, "Experimental backend is disabled");
    }
    let plan = match make_plan_with_scope(exe, extended) {
        Ok(plan) if plan.supported => plan,
        Ok(_) => return english(exe, &arguments, "Unsupported official package hash"),
        Err(_) => return english(exe, &arguments, "Cannot prepare a compatible patch plan"),
    };
    let operation = (|| -> Result<Value> {
        let mut process = Process::spawn_suspended(exe, &arguments)?;
        let base = process.image_base()?;
        let count = apply(&process, base, &plan, None)?;
        process.resume()?;
        Ok(
            json!({"mode":"experimental-launch","pid":process.pid,"count":count,"resident_launcher":false,
            "memory_left_applied":true,"sha256":plan.sha256,"extended":extended,"isolated":isolated}),
        )
    })();
    operation.or_else(|_| {
        english(
            exe,
            &arguments,
            "Experimental backend failed; suspended child was ended",
        )
    })
}

#[cfg(windows)]
fn restore_pid(exe: &Path, pid: u32, extended: bool) -> Result<Value> {
    use zed_zh_addon::windows_runtime::{Process, restore};
    let plan = make_plan_with_scope(exe, extended)?;
    ensure!(plan.supported, "Unsupported executable");
    let process = Process::open(pid)?;
    process.verify_identity(exe)?;
    let base = process.image_base()?;
    let restored = restore(&process, base, &plan)?;
    Ok(json!({"mode":"restore","pid":pid,"restored":restored,"verified":true}))
}

fn run(cli: Cli) -> Result<Value> {
    #[cfg(feature = "ai")]
    match &cli.command {
        Command::Translate {
            jobs,
            config,
            cache,
            toolkit,
            python,
            bridge,
            refresh,
        } => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            return runtime.block_on(zed_zh_addon::translation::translate(
                jobs, config, cache, toolkit, python, bridge, *refresh,
            ));
        }
        Command::CheckUpdate => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            return runtime.block_on(zed_zh_addon::translation::check_update());
        }
        _ => {}
    }
    let exe = discover_exe(cli.exe)?;
    match cli.command {
        Command::Plan => Ok(serde_json::to_value(make_plan_with_scope(
            &exe,
            cli.extended,
        )?)?),
        #[cfg(windows)]
        Command::TestMemory => memory_test(&exe, cli.extended),
        #[cfg(windows)]
        Command::Launch {
            experimental,
            isolated,
            arguments,
        } => launch(&exe, experimental, isolated, arguments, cli.extended),
        #[cfg(windows)]
        Command::Restore { pid } => restore_pid(&exe, pid, cli.extended),
        #[allow(unreachable_patterns)]
        _ => bail!("Process operations are only supported on Windows x64"),
    }
}

fn main() {
    let cli = Cli::parse();
    let output = cli.output.clone();
    let started = Instant::now();
    match run(cli) {
        Ok(mut report) => {
            report["tool_elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
            let text = serde_json::to_string_pretty(&report).expect("JSON report") + "\n";
            if let Some(path) = output {
                let report_path = path.display().to_string();
                let written = (|| -> Result<()> {
                    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty())
                    {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(path, &text)?;
                    Ok(())
                })();
                if let Err(error) = written {
                    eprintln!("Cannot save report: {error}");
                    std::process::exit(1);
                }
                if let Some(object) = report.as_object_mut() {
                    object.remove("entries");
                    object.remove("translations");
                    object.insert("report_file".into(), json!(report_path));
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).expect("JSON summary")
                );
            } else {
                print!("{text}");
            }
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}
