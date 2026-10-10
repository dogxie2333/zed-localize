#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
fn main() {
    #[cfg(windows)]
    if run().is_err() {
        // Provider response bodies and credentials never reach error dialogs.
        use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
        let message: Vec<u16> = "设置服务未能启动，请检查目录权限和本机端口。"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let title: Vec<u16> = "Zed".encode_utf16().chain(Some(0)).collect();
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    eprintln!("The settings component requires Windows.");
}
#[cfg(windows)]
fn run() -> anyhow::Result<()> {
    use std::path::PathBuf;
    let root = PathBuf::from(
        std::env::var_os("ZED_ZH_RUNTIME_ROOT")
            .ok_or_else(|| anyhow::anyhow!("Missing resource directory"))?,
    );
    let state = PathBuf::from(
        std::env::var_os("ZED_ZH_STATE_DIR")
            .ok_or_else(|| anyhow::anyhow!("Missing state directory"))?,
    );
    let launcher = PathBuf::from(
        std::env::var_os("ZED_ZH_LAUNCHER").ok_or_else(|| anyhow::anyhow!("Missing launcher"))?,
    );
    let args: Vec<_> = std::env::args().collect();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        if args.iter().any(|arg| arg == "--maintenance") {
            zed_zh_addon::configuration::automatic(&root, &state).await
        } else {
            zed_zh_addon::configuration::serve(
                root,
                state,
                launcher,
                !args.iter().any(|arg| arg == "--no-open"),
            )
            .await
        }
    })
}
