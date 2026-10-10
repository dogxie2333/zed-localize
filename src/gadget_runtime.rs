//! Native display glue: mature DLL injection crate + official Frida Gadget.
use crate::{
    catalog::{embedded_catalog, hash_file},
    windows_runtime::Process,
};
use anyhow::{Context, Result, ensure};
use dll_syringe::{Syringe, process::OwnedProcess};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

pub fn config(root: &Path, exe: &Path, profile: Option<&Path>) -> Result<Option<Value>> {
    let digest = hash_file(exe)?;
    let mut adapters: Value =
        serde_json::from_slice(&fs::read(root.join("data/ui-adapters.json"))?)?;
    let state = state_dir(root);
    if let Ok(bytes) = fs::read(state.join("verified-adapters.json")) {
        let updates: Value = serde_json::from_slice(&bytes)?;
        if let Some(items) = updates["adapters"].as_array() {
            adapters["adapters"]
                .as_array_mut()
                .context("Invalid adapter list")?
                .extend(items.iter().cloned());
        }
    }
    let Some(adapter) = adapters["adapters"]
        .as_array()
        .context("Invalid adapter list")?
        .iter()
        .find(|item| item["verified"] == true && item["sha256"] == digest)
    else {
        return Ok(None);
    };
    let catalog = embedded_catalog()?;
    let messages: serde_json::Map<String, Value> = catalog
        .messages
        .iter()
        .filter(|item| {
            item.source != item.translation
                && !item
                    .source
                    .chars()
                    .chain(item.translation.chars())
                    .any(|c| "{}\n\r\t`*_[]<>".contains(c))
                && item.translation.encode_utf16().count() <= item.source.encode_utf16().count()
        })
        .map(|item| (item.source.clone(), json!(item.translation)))
        .collect();
    let bytes = fs::read(exe)?;
    let pe = goblin::pe::PE::parse(&bytes)?;
    let size = pe
        .header
        .optional_header
        .context("Missing PE optional header")?
        .windows_fields
        .size_of_image;
    let mut fonts = adapter["default_ui_fonts"]
        .as_array()
        .context("Missing UI fonts")?
        .clone();
    let config_path = if let Some(profile) = profile {
        profile.join("config/settings.json")
    } else {
        PathBuf::from(std::env::var_os("APPDATA").context("Missing user profile")?)
            .join("Zed/settings.json")
    };
    if let Ok(text) = fs::read_to_string(config_path)
        && let Ok(settings) = json5::from_str::<Value>(&text)
        && let Some(font) = settings["ui_font_family"].as_str()
    {
        fonts.push(json!(font));
    }
    Ok(Some(
        json!({"messages":messages,"ui_fonts":fonts,"ui_caller_rva":adapter["ui_caller_rva"],"image_size":size,"protected_names":[],"sha256":digest}),
    ))
}
pub fn state_dir(root: &Path) -> PathBuf {
    crate::runtime_paths::state(root)
}

pub fn attach(
    root: &Path,
    exe: &Path,
    pid: u32,
    profile: Option<&Path>,
    existing: bool,
) -> Result<bool> {
    attach_inner(root, exe, pid, profile, existing, None)
}
fn attach_inner(
    root: &Path,
    exe: &Path,
    pid: u32,
    profile: Option<&Path>,
    existing: bool,
    test_code: Option<&str>,
) -> Result<bool> {
    let process = Process::open(pid)?;
    process.verify_identity(exe)?;
    let Some(mut config) = config(root, exe, profile)? else {
        return Ok(false);
    };
    let folder = state_dir(root).join(format!("gadgets/{pid}-{}", now()));
    fs::create_dir_all(&folder)?;
    let dll = folder.join("zed-zh-gadget.dll");
    let vendor = root.join("vendor/frida-gadget-17.23.3.dll");
    let vendor = if vendor.is_file() {
        vendor
    } else {
        root.join("local/vendor/frida-gadget-17.23.3.dll")
    };
    if fs::hard_link(&vendor, &dll).is_err() {
        fs::copy(vendor, &dll)?;
    }
    config["attach_existing"] = json!(existing);
    fs::create_dir_all(state_dir(root).join("gadget-children"))?;
    let parameters = json!({"config":config,"stats_path":folder.join("stats.json"),"control_path":state_dir(root).join("gadget-control.json"),"child_request_dir":state_dir(root).join("gadget-children"),"profile":profile,"launcher_path":std::env::current_exe()?});
    let script = folder.join("overlay.js");
    let source = format!(
        "{}\n{}",
        fs::read_to_string(root.join("tools/ui_overlay.js"))?,
        fs::read_to_string(root.join("tools/gadget_lifecycle.js"))?
    );
    let wrapper = r#"
rpc.exports.init=function(stage,parameters){
  try {
    rpc.exports.configure(parameters.config);
    installRestartBridge(parameters);
    const temporary=parameters.stats_path+'.tmp';
    const move=new NativeFunction(Process.getModuleByName('kernel32.dll').getExportByName('MoveFileExW'),'int',['pointer','pointer','uint']);
    const temporaryPath=Memory.allocUtf16String(temporary);
    const statsPath=Memory.allocUtf16String(parameters.stats_path);
    const record=()=>{
      try {const control=JSON.parse(File.readAllText(parameters.control_path));enabled=control.enabled!==false;}catch(_){}
      const snapshot={pid:Process.id,mode:enabled?'chinese':'paused',dictionary_count:dictionary.size,translated_distinct:hits.size,updated:Date.now()/1000};
      const file=new File(temporary,'w');file.write(JSON.stringify(snapshot));file.flush();file.close();
      move(temporaryPath,statsPath,1);
    };
    record();setInterval(record,2000);
  } catch (_) {File.writeAllText(parameters.stats_path,JSON.stringify({pid:Process.id,mode:'failed',updated:Date.now()/1000}));}
};
"#;
    fs::write(
        script,
        format!("{source}\n{wrapper}\n{}", test_code.unwrap_or("")),
    )?;
    fs::write(
        dll.with_extension("config"),
        serde_json::to_vec(
            &json!({"interaction":{"type":"script","path":folder.join("overlay.js"),"parameters":parameters},"runtime":"qjs","teardown":"full"}),
        )?,
    )?;
    let target = OwnedProcess::from_pid(pid)?;
    let syringe = Syringe::for_process(target);
    syringe.inject(dll)?;
    Ok(true)
}

pub fn test(root: &Path, exe: &Path) -> Result<Value> {
    enable(root, true)?;
    let profile = state_dir(root).join("native-gadget-test");
    fs::create_dir_all(profile.join("config"))?;
    fs::write(
        profile.join("config/settings.json"),
        r#"{"auto_update":false,"telemetry":{"metrics":false,"diagnostics":false}}"#,
    )?;
    let saved = std::env::var_os("ZED_STATELESS");
    unsafe {
        std::env::set_var("ZED_STATELESS", "1");
    }
    let spawned = Process::spawn_suspended(
        exe,
        &[
            "--user-data-dir".into(),
            profile.to_string_lossy().into_owned(),
        ],
    );
    unsafe {
        if let Some(saved) = saved {
            std::env::set_var("ZED_STATELESS", saved);
        } else {
            std::env::remove_var("ZED_STATELESS");
        }
    }
    let mut process = spawned?;
    let pid = process.pid;
    let owner = state_dir(root).join(format!("test-child-{pid}.json"));
    let command = format!(
        "\"{}\" --user-data-dir \"{}\"",
        exe.display(),
        profile.display()
    );
    let trigger = fs::read_to_string(root.join("tools/test_gadget_restart.js"))?
        .replace("__COMMAND__", &serde_json::to_string(&command)?)
        .replace(
            "__OWNER_PATH__",
            &serde_json::to_string(&owner.to_string_lossy())?,
        );
    let result = (|| -> Result<Value> {
        ensure!(
            attach_inner(root, exe, pid, Some(&profile), false, Some(&trigger))?,
            "Unsupported binary"
        );
        process.resume()?;
        let mut parent_report = None;
        for _ in 0..32 {
            handle_children(root, exe)?;
            if parent_report.is_none() && owner.is_file() {
                parent_report = snapshots(root, exe)
                    .into_iter()
                    .find(|item| item["pid"].as_u64() == Some(u64::from(pid)));
                let _ = process.terminate_owned();
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        let child =
            serde_json::from_slice::<Value>(&fs::read(state_dir(root).join("last-launch.json"))?)?
                ["pid"]
                .as_u64()
                .context("Missing self-test child")?;
        let mut reports: Vec<Value> = snapshots(root, exe)
            .into_iter()
            .filter(|item| {
                item["pid"]
                    .as_u64()
                    .is_some_and(|id| id == u64::from(pid) || id == child)
            })
            .collect();
        if let Some(parent) = parent_report {
            reports.push(parent);
        }
        ensure!(
            reports.len() == 2
                && reports
                    .iter()
                    .all(|item| item["translated_distinct"].as_u64().unwrap_or_default() > 0),
            "Parent/child Chinese rendering was not observed"
        );
        Ok(
            json!({"passed":true,"snapshots":reports,"python_runtime":false,"self_restart_via_launcher":true}),
        )
    })();
    if let (Ok(owner_bytes), Ok(launch_bytes)) = (
        fs::read(&owner),
        fs::read(state_dir(root).join("last-launch.json")),
    ) && let (Ok(owner), Ok(launch)) = (
        serde_json::from_slice::<Value>(&owner_bytes),
        serde_json::from_slice::<Value>(&launch_bytes),
    ) && owner["parent_pid"].as_u64() == Some(u64::from(pid))
        && owner["pid"] == launch["launcher_pid"]
        && let (Some(child), Some(frontend)) = (
            launch["pid"].as_u64().and_then(|id| u32::try_from(id).ok()),
            owner["pid"].as_u64().and_then(|id| u32::try_from(id).ok()),
        )
    {
        let _ = process.terminate_owned_descendant(child, frontend, exe);
    }
    let terminated = process.terminate_owned();
    let report = result?;
    let _ = terminated;
    Ok(report)
}

pub fn enable(root: &Path, enabled: bool) -> Result<()> {
    let folder = state_dir(root);
    fs::create_dir_all(&folder)?;
    let temporary = folder.join("gadget-control.tmp");
    fs::write(&temporary, serde_json::to_vec(&json!({"enabled":enabled}))?)?;
    fs::rename(temporary, folder.join("gadget-control.json"))?;
    Ok(())
}

pub fn snapshots(root: &Path, exe: &Path) -> Vec<Value> {
    let mut reports = std::collections::BTreeMap::new();
    if let Ok(folders) = fs::read_dir(state_dir(root).join("gadgets")) {
        for folder in folders.flatten() {
            let name = folder.file_name();
            let Some(pid) = name
                .to_string_lossy()
                .split('-')
                .next()
                .and_then(|s| s.parse::<u32>().ok())
            else {
                continue;
            };
            if reports.contains_key(&pid) {
                continue;
            }
            let Ok(process) = Process::open(pid) else {
                continue;
            };
            if process.verify_identity(exe).is_err() {
                continue;
            }
            if let Ok(bytes) = fs::read(folder.path().join("stats.json"))
                && let Ok(report) = serde_json::from_slice::<Value>(&bytes)
                && now() - report["updated"].as_f64().unwrap_or_default() < 10.0
            {
                reports.insert(pid, report);
            }
        }
    }
    reports.into_values().collect()
}

pub fn handle_children(root: &Path, exe: &Path) -> Result<()> {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            EVENT_MODIFY_STATE, GetProcessIdOfThread, OpenEventW, OpenThread, ResumeThread,
            SetEvent, THREAD_QUERY_LIMITED_INFORMATION, THREAD_SUSPEND_RESUME,
        },
    };
    let folder = state_dir(root).join("gadget-children");
    let Ok(entries) = fs::read_dir(&folder) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let Some(request) = fs::read(&path)
            .ok()
            .and_then(|data| serde_json::from_slice::<Value>(&data).ok())
        else {
            continue;
        };
        if request["kind"] != "zed-self-restart" {
            continue;
        }
        let Some(pid) = request["pid"].as_u64().and_then(|n| u32::try_from(n).ok()) else {
            continue;
        };
        let Some(parent) = request["parent_pid"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
        else {
            continue;
        };
        let Some(tid) = request["thread_id"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
        else {
            continue;
        };
        let Ok(parent_process) = Process::open(parent) else {
            let _ = fs::remove_file(path);
            continue;
        };
        if parent_process.verify_identity(exe).is_err() {
            continue;
        }
        let Ok(child) = Process::open(pid) else {
            let _ = fs::remove_file(path);
            continue;
        };
        if child.verify_identity(exe).is_err() {
            continue;
        }
        let thread = unsafe {
            OpenThread(
                THREAD_SUSPEND_RESUME | THREAD_QUERY_LIMITED_INFORMATION,
                0,
                tid,
            )
        };
        if thread.is_null() {
            continue;
        }
        if unsafe { GetProcessIdOfThread(thread) } != pid {
            unsafe {
                CloseHandle(thread);
            }
            continue;
        }
        let profile = request["profile"].as_str().map(Path::new);
        let _ = attach(root, exe, pid, profile, true);
        unsafe {
            ResumeThread(thread);
            CloseHandle(thread);
        }
        if let Some(name) = request["event"]
            .as_str()
            .filter(|name| name.starts_with("Local\\ZedZhResume-"))
        {
            let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
            let event = unsafe { OpenEventW(EVENT_MODIFY_STATE, 0, wide.as_ptr()) };
            if !event.is_null() {
                unsafe {
                    SetEvent(event);
                    CloseHandle(event);
                }
            }
        }
        let _ = fs::remove_file(path);
    }
    Ok(())
}
