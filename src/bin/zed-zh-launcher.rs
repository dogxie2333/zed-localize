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
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
            },
        },
    };
    use zed_zh_addon::runtime_paths::state;
    const HIDDEN: u32 = 0x0800_0000;

    fn root() -> Result<PathBuf> {
        #[cfg(feature = "portable")]
        return zed_zh_addon::portable::root();
        #[cfg(not(feature = "portable"))]
        {
            let exe = std::env::current_exe()?;
            exe.ancestors()
                .find(|path| path.join("data/ui-adapters.json").is_file())
                .map(Path::to_owned)
                .context("Cannot locate local runtime")
        }
    }
    fn spawn_settings(root: &Path, maintenance: bool) -> Result<()> {
        fs::create_dir_all(state(root).join("tmp"))?;
        let mut command = Command::new(root.join("zed-zh-settings.exe"));
        if maintenance {
            command.arg("--maintenance");
        }
        if std::env::var("ZED_ZH_NO_OPEN").as_deref() == Ok("1") {
            command.arg("--no-open");
        }
        command
            .current_dir(root)
            .env_remove("ZED_I18N_API_KEY")
            .env("ZED_ZH_RUNTIME_ROOT", root)
            .env("ZED_ZH_STATE_DIR", state(root))
            .env("ZED_ZH_LAUNCHER", std::env::current_exe()?)
            .env("TEMP", state(root).join("tmp"))
            .env("TMP", state(root).join("tmp"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(HIDDEN)
            .spawn()?;
        Ok(())
    }
    fn settings(root: &Path) -> Value {
        fs::read(state(root).join("settings.json"))
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
            .env("ZED_ZH_STATE_DIR", state(root))
            .env("ZED_ZH_LAUNCHER", std::env::current_exe()?)
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
        let Some(name) = Path::new(exe).file_name().and_then(|name| name.to_str()) else {
            return vec![];
        };
        // Filter by executable name before opening process handles. Full-path
        // verification remains mandatory, but unrelated processes cost no query.
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return vec![];
        }
        let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut pids = Vec::new();
        let mut available = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
        while available {
            let length = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..length]).eq_ignore_ascii_case(name) {
                pids.push(entry.th32ProcessID);
            }
            available = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
        }
        unsafe {
            CloseHandle(snapshot);
        }
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
    fn watch_worker(root: &Path, stop: &AtomicBool, health: &mpsc::Sender<bool>) -> Result<()> {
        while !state(root).join("settings.json").is_file() || settings(root)["initialized"] == false
        {
            if stop.load(Ordering::Relaxed) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        let enabled = settings(root)["watch_enabled"].as_bool() != Some(false);
        zed_zh_addon::gadget_runtime::enable(root, enabled)?;
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |event| {
                let _ = tx.send(event);
            },
            notify::Config::default(),
        )?;
        let current = settings(root);
        let mut watched_exe = current["zed_exe"]
            .as_str()
            .unwrap_or("D:\\Zed\\Zed.exe")
            .to_owned();
        let mut roots = vec![
            Path::new(&watched_exe)
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
        let children = state(root).join("gadget-children");
        fs::create_dir_all(&children)?;
        watcher.watch(&children, RecursiveMode::NonRecursive)?;
        repair(root)?;
        let _ = health.send(false);
        let mut seen = HashSet::new();
        let mut dirty = false;
        let mut last_repair = Instant::now();
        let mut last_scan = Instant::now() - Duration::from_secs(5);
        while !stop.load(Ordering::Relaxed) {
            if rx.recv_timeout(Duration::from_secs(5)).is_ok() {
                dirty = true;
            }
            while rx.try_recv().is_ok() {
                dirty = true;
            }
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let child_config = settings(root);
            let child_exe = Path::new(
                child_config["zed_exe"]
                    .as_str()
                    .unwrap_or("D:\\Zed\\Zed.exe"),
            );
            let _ = zed_zh_addon::gadget_runtime::handle_children(root, child_exe);
            if dirty && last_repair.elapsed() > Duration::from_secs(5) {
                repair(root)?;
                let _ = Command::new(std::env::current_exe()?)
                    .arg("--update-compat")
                    .creation_flags(HIDDEN)
                    .spawn();
                dirty = false;
                last_repair = Instant::now();
            }
            // Patched shortcuts launch with the display engine immediately.
            // Polling is only recovery for starting the original executable.
            let scan_interval = if seen.is_empty() { 30 } else { 10 };
            if last_scan.elapsed() < Duration::from_secs(scan_interval) {
                continue;
            }
            last_scan = Instant::now();
            let config = settings(root);
            let exe = config["zed_exe"].as_str().unwrap_or("D:\\Zed\\Zed.exe");
            if watched_exe != exe {
                if let Some(parent) = Path::new(exe).parent().filter(|path| path.is_dir()) {
                    watcher.watch(parent, RecursiveMode::Recursive)?;
                }
                watched_exe = exe.to_owned();
                seen.clear();
            }
            let next: HashSet<_> = zed_pids(exe).into_iter().collect();
            if config["watch_enabled"].as_bool() != Some(false) {
                for pid in next.difference(&seen) {
                    let _ = Command::new(std::env::current_exe()?)
                        .args(["--attach", &pid.to_string()])
                        .creation_flags(HIDDEN)
                        .spawn();
                }
            }
            let snapshots = zed_zh_addon::gadget_runtime::snapshots(root, Path::new(exe));
            write_json(
                &state(root).join("runtime-status.json"),
                &json!({"controller_pid":std::process::id(),"engine":"rust-frida-gadget","sessions":snapshots,"updated":now()}),
            )?;
            seen = next;
        }
        Ok(())
    }

    fn now() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }
    fn write_json(path: &Path, value: &Value) -> Result<()> {
        fs::create_dir_all(path.parent().context("No JSON parent")?)?;
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
        fs::rename(temporary, path)?;
        Ok(())
    }
    fn stop_controller(root: &Path) -> Result<()> {
        zed_zh_addon::gadget_runtime::enable(root, false)
    }
    fn taskbar_dark() -> bool {
        use windows_sys::Win32::System::Registry::{
            HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW,
        };
        let key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let name: Vec<u16> = "SystemUsesLightTheme"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut light = 0u32;
        let mut length = 4u32;
        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                (&mut light as *mut u32).cast(),
                &mut length,
            )
        };
        result != 0 || light == 0
    }
    fn icon(state: &str, dark: bool) -> Result<tray_icon::Icon> {
        // Pre-rendered RGBA avoids an image decoder or Python in the tray.
        macro_rules! asset {
            ($name:literal) => {
                include_bytes!(concat!("../../assets/icons/tray-", $name, ".rgba")) as &[u8]
            };
        }
        let pixels = match (dark, state) {
            (false, "active") => asset!("light-active"),
            (false, "paused") => asset!("light-paused"),
            (false, "unsupported") => asset!("light-unsupported"),
            (false, "error") => asset!("light-error"),
            (false, _) => asset!("light-waiting"),
            (true, "active") => asset!("dark-active"),
            (true, "paused") => asset!("dark-paused"),
            (true, "unsupported") => asset!("dark-unsupported"),
            (true, "error") => asset!("dark-error"),
            (true, _) => asset!("dark-waiting"),
        };
        Ok(tray_icon::Icon::from_rgba(pixels.to_vec(), 32, 32)?)
    }
    fn tray_state(root: &Path, failed: bool) -> (&'static str, &'static str) {
        if failed {
            return ("监控异常：请打开配置页面", "error");
        }
        if settings(root)["watch_enabled"].as_bool() == Some(false) {
            return ("已暂停自动汉化", "paused");
        }
        let status = fs::read(state(root).join("runtime-status.json"))
            .ok()
            .and_then(|data| serde_json::from_slice::<Value>(&data).ok());
        if let Some(status) =
            status.filter(|item| now() - item["updated"].as_f64().unwrap_or_default() < 15.0)
            && let Some(sessions) = status["sessions"].as_array()
        {
            if sessions.iter().any(|item| item["mode"] == "chinese") {
                return ("汉化已启用：Zed 运行中", "active");
            }
            if sessions
                .iter()
                .any(|item| item["mode"] == "unsupported-english")
            {
                return ("版本未适配：当前为英文", "unsupported");
            }
        }
        ("补丁待命：等待 Zed 启动", "waiting")
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
        if !state(root).join("settings.json").is_file() || settings(root)["initialized"] == false {
            let _ = spawn_settings(root, false);
        }
        use tray_icon::{
            TrayIconBuilder,
            menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, MSG, MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx, PM_REMOVE,
            PeekMessageW, QS_ALLINPUT, TranslateMessage,
        };
        let menu = Menu::new();
        let status_item = MenuItem::new("补丁正在启动", false, None);
        let configure = MenuItem::new("打开配置网页…", true, None);
        let close_settings = MenuItem::new("关闭配置服务（保留汉化）", true, None);
        let launch = MenuItem::new("启动中文 Zed", true, None);
        let pause = MenuItem::new("暂停自动汉化", true, None);
        let exit = MenuItem::new("退出补丁（不关闭 Zed）", true, None);
        menu.append_items(&[
            &status_item,
            &PredefinedMenuItem::separator(),
            &configure,
            &close_settings,
            &launch,
            &pause,
            &PredefinedMenuItem::separator(),
            &exit,
        ])?;
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("Zed 汉化补丁：正在启动")
            .with_icon(icon("waiting", taskbar_dark())?)
            .build()?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let worker_root = root.to_owned();
        let (error_tx, error_rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                if watch_worker(&worker_root, &worker_stop, &error_tx).is_err() {
                    let _ = error_tx.send(true);
                    // Updates may temporarily replace files or remove a watched
                    // directory. Recover without spinning or losing the tray.
                    for _ in 0..20 {
                        if worker_stop.load(Ordering::Relaxed) {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(250));
                    }
                }
            }
        });
        let mut failed = false;
        let mut displayed = String::new();
        let opened = now();
        let mut last_status = Instant::now() - Duration::from_secs(2);
        let mut last_menu_request = opened;
        loop {
            let mut message: MSG = unsafe { std::mem::zeroed() };
            while unsafe { PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0
            {
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == exit.id() {
                    stop.store(true, Ordering::Relaxed);
                } else if event.id == configure.id() {
                    let _ = spawn_settings(root, false);
                } else if event.id == close_settings.id() {
                    let _ = write_json(
                        &state(root).join("settings-stop.json"),
                        &json!({"time":now()}),
                    );
                } else if event.id == launch.id() {
                    let _ = Command::new(std::env::current_exe()?)
                        .creation_flags(HIDDEN)
                        .spawn();
                } else if event.id == pause.id() {
                    let mut config = settings(root);
                    let enable = config["watch_enabled"].as_bool() == Some(false);
                    config["watch_enabled"] = json!(enable);
                    write_json(&state(root).join("settings.json"), &config)?;
                    if enable {
                        for pid in
                            zed_pids(config["zed_exe"].as_str().unwrap_or("D:\\Zed\\Zed.exe"))
                        {
                            let _ = Command::new(std::env::current_exe()?)
                                .args(["--attach", &pid.to_string()])
                                .creation_flags(HIDDEN)
                                .spawn();
                        }
                    } else {
                        let _ = stop_controller(root);
                    }
                    let _ = zed_zh_addon::gadget_runtime::enable(root, enable);
                }
            }
            if last_status.elapsed() >= Duration::from_secs(5) {
                while let Ok(value) = error_rx.try_recv() {
                    failed = value;
                }
                let (text, mark) = tray_state(root, failed);
                let dark = taskbar_dark();
                let display_key = format!("{text}:{dark}");
                if displayed != display_key {
                    tray.set_tooltip(Some(format!("Zed 汉化补丁：{text}")))?;
                    tray.set_icon(Some(icon(mark, dark)?))?;
                    status_item.set_text(text);
                    pause.set_text(
                        if settings(root)["watch_enabled"].as_bool() == Some(false) {
                            "恢复自动汉化"
                        } else {
                            "暂停自动汉化"
                        },
                    );
                    displayed = display_key;
                }
                write_json(
                    &state(root).join("tray-status.json"),
                    &json!({"pid":std::process::id(),"state":text,"running":true,"registered":true,"shell_icon_visible":tray.rect().is_some(),"icon_theme":if dark {"dark"}else{"light"},"icon_mark":mark,"updated":now()}),
                )?;
                last_status = Instant::now();
                let request = fs::read(state(root).join("tray-stop.json"))
                    .ok()
                    .and_then(|data| serde_json::from_slice::<Value>(&data).ok());
                if request.is_some_and(|item| item["time"].as_f64().unwrap_or_default() >= opened) {
                    stop.store(true, Ordering::Relaxed);
                }
                let request = fs::read(state(root).join("show-menu.json"))
                    .ok()
                    .and_then(|data| serde_json::from_slice::<Value>(&data).ok());
                if let Some(time) = request
                    .and_then(|item| item["time"].as_f64())
                    .filter(|time| *time > last_menu_request)
                {
                    last_menu_request = time;
                    use tray_icon::menu::ContextMenu;
                    unsafe {
                        menu.show_context_menu_for_hwnd(tray.window_handle() as isize, None);
                    }
                }
            }
            if stop.load(Ordering::Relaxed) {
                break;
            }
            unsafe {
                MsgWaitForMultipleObjectsEx(
                    0,
                    std::ptr::null(),
                    5000,
                    QS_ALLINPUT,
                    MWMO_INPUTAVAILABLE,
                );
            }
        }
        stop.store(true, Ordering::Relaxed);
        let _ = worker.join();
        let _ = stop_controller(root);
        let _ = write_json(
            &state(root).join("settings-stop.json"),
            &json!({"time":now()}),
        );
        let _ = write_json(
            &state(root).join("tray-status.json"),
            &json!({"pid":std::process::id(),"state":"补丁已退出","running":false,"updated":now()}),
        );
        drop(tray);
        unsafe {
            CloseHandle(mutex);
        }
        Ok(())
    }
    pub fn run() -> Result<()> {
        let root = root()?;
        let args: Vec<String> = std::env::args().skip(1).collect();
        if args
            .first()
            .is_some_and(|value| value == "--test-native-overlay")
        {
            let config = settings(&root);
            let report = zed_zh_addon::gadget_runtime::test(
                &root,
                Path::new(config["zed_exe"].as_str().unwrap_or("D:\\Zed\\Zed.exe")),
            )?;
            write_json(&root.join("reports/native-gadget.json"), &report)?;
            return Ok(());
        }
        if args.first().is_some_and(|value| value == "--watch") {
            return watch(&root);
        }
        if args.first().is_some_and(|value| value == "--settings") {
            return spawn_settings(&root, false);
        }
        if args.first().is_some_and(|value| value == "--attach") {
            let pid = args.get(1).context("Missing PID")?.parse::<u32>()?;
            let config = settings(&root);
            let exe = Path::new(config["zed_exe"].as_str().unwrap_or("D:\\Zed\\Zed.exe"));
            if !zed_zh_addon::gadget_runtime::snapshots(&root, exe)
                .iter()
                .any(|item| item["pid"].as_u64() == Some(pid as u64))
            {
                zed_zh_addon::gadget_runtime::attach(&root, exe, pid, None, true)?;
            }
            return Ok(());
        }
        if args.first().is_some_and(|value| value == "--update-compat") {
            #[cfg(feature = "ai")]
            return update_compat(&root);
            #[cfg(not(feature = "ai"))]
            return Ok(());
        }
        if args.first().is_some_and(|value| value == "--stop") {
            return write_json(&state(&root).join("tray-stop.json"), &json!({"time":now()}));
        }
        if args.first().is_some_and(|value| value == "--show-menu") {
            return write_json(&state(&root).join("show-menu.json"), &json!({"time":now()}));
        }
        let _ = Command::new(std::env::current_exe()?)
            .arg("--watch")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(HIDDEN)
            .spawn();
        if !state(&root).join("settings.json").is_file() || settings(&root)["initialized"] == false
        {
            return Ok(());
        }
        let mut config = settings(&root);
        let english = args.first().is_some_and(|value| value == "--english");
        config["watch_enabled"] = json!(!english);
        write_json(&state(&root).join("settings.json"), &config)?;
        zed_zh_addon::gadget_runtime::enable(&root, !english)?;
        let exe = Path::new(config["zed_exe"].as_str().unwrap_or("D:\\Zed\\Zed.exe"));
        let arguments = if english { &args[1..] } else { &args[..] };
        let mut process = zed_zh_addon::windows_runtime::Process::spawn_suspended(exe, arguments)?;
        let mode = if english {
            "english"
        } else {
            match zed_zh_addon::gadget_runtime::attach(&root, exe, process.pid, None, false) {
                Ok(true) => "chinese",
                Ok(false) => "unsupported-english",
                Err(_) => "attach-failed-english",
            }
        };
        process.resume()?;
        write_json(
            &state(&root).join("last-launch.json"),
            &json!({"pid":process.pid,"launcher_pid":std::process::id(),"mode":mode,"updated":now()}),
        )?;
        Ok(())
    }

    #[cfg(feature = "ai")]
    fn update_compat(root: &Path) -> Result<()> {
        let stamp = state(root).join("compat-last-check.json");
        if let Ok(bytes) = fs::read(&stamp)
            && let Ok(data) = serde_json::from_slice::<Value>(&bytes)
            && now() - data["time"].as_f64().unwrap_or_default() < 3600.0
        {
            return Ok(());
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let result=runtime.block_on(async {
            let client=reqwest::Client::builder().timeout(Duration::from_secs(20)).user_agent("zed-zh-addon-native-maintenance").build()?;
            let data:Value=client.get("https://raw.githubusercontent.com/dogxie2333/zed-zh-addon/main/data/ui-adapters.json").send().await?.error_for_status()?.json().await?;
            anyhow::ensure!(data["schema"]==1,"Unexpected adapter schema");
            for adapter in data["adapters"].as_array().context("Missing adapter list")? {
                let hash=adapter["sha256"].as_str().context("Invalid adapter hash")?;
                anyhow::ensure!(hash.len()==64 && hash.bytes().all(|c|c.is_ascii_hexdigit()) && adapter["verified"].is_boolean(),"Invalid adapter data");
            }
            Ok::<Value,anyhow::Error>(data)
        });
        match result {
            Ok(data) => {
                write_json(&state(root).join("verified-adapters.json"), &data)?;
                write_json(
                    &stamp,
                    &json!({"time":now(),"status":"checked","adapter_count":data["adapters"].as_array().map_or(0,Vec::len)}),
                )?;
            }
            Err(_) => {
                write_json(&stamp, &json!({"time":now(),"status":"check-failed"}))?;
            }
        }
        if settings(root)["auto_translate"] == true {
            spawn_settings(root, true)?;
        }
        Ok(())
    }
}

fn main() {
    #[cfg(windows)]
    if let Err(error) = desktop::run() {
        if !std::env::args()
            .any(|arg| matches!(arg.as_str(), "--attach" | "--update-compat" | "--stop"))
        {
            use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
            let message: Vec<u16> =
                format!("启动未完成：{error}\n请保留 EXE 和缓存目录，并确认目录可写、空间充足。")
                    .encode_utf16()
                    .chain(Some(0))
                    .collect();
            let title: Vec<u16> = "Zed 汉化补丁".encode_utf16().chain(Some(0)).collect();
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    message.as_ptr(),
                    title.as_ptr(),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
        eprintln!("{error:#}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("The desktop launcher requires Windows");
        std::process::exit(1);
    }
}
