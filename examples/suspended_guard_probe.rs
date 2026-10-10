//! Local integration fixture for the kernel-owned suspended-child guard.
use anyhow::{Context, Result};

#[cfg(windows)]
fn main() -> Result<()> {
    use std::io::Write;
    use zed_zh_addon::windows_runtime::Process;
    let executable = std::env::args_os()
        .nth(1)
        .context("Missing test executable")?;
    let profile = std::env::args_os()
        .nth(2)
        .context("Missing isolated profile")?;
    let process = Process::spawn_suspended(
        std::path::Path::new(&executable),
        &[
            "--user-data-dir".into(),
            profile.to_string_lossy().into_owned(),
        ],
    )?;
    println!("{}", process.pid);
    std::io::stdout().flush()?;
    // The parent test forcibly ends this fixture. The job handle must close
    // in the kernel and end only this fixture's own paused Zed child.
    std::thread::park();
    Ok(())
}

#[cfg(not(windows))]
fn main() -> Result<()> {
    let _ = std::env::args_os()
        .nth(1)
        .context("Missing test executable")?;
    anyhow::bail!("Windows-only integration fixture")
}
