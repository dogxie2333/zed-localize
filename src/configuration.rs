//! On-demand native settings. Axum, async-openai, keyring and rfd provide the
//! infrastructure; this module only binds the companion's existing workflows.
use anyhow::{Context, Result, ensure};
use async_openai::{Client, config::OpenAIConfig};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tower_http::services::ServeDir;

const HIDDEN: u32 = 0x0800_0000;
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn read(path: &Path) -> Option<Value> {
    fs::read(path).ok().and_then(|bytes| {
        serde_json::from_slice(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes)).ok()
    })
}
fn defaults() -> Value {
    json!({"zed_exe":"D:\\Zed\\Zed.exe","watch_enabled":true,"startup_mode":"manual","replace_shortcuts":true,"provider":"deepseek","api_base":"https://api.deepseek.com/v1","model":"deepseek-flash","auto_translate":false,"ui_language":"zh-CN"})
}
fn settings(state: &Path) -> Value {
    let mut value = defaults();
    if let Some(Value::Object(saved)) = read(&state.join("settings.json")) {
        for (key, item) in saved {
            value[key] = item;
        }
    }
    value
}
fn write(path: &Path, value: &Value) -> Result<()> {
    fs::create_dir_all(path.parent().context("No parent")?)?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    fs::rename(temporary, path)?;
    Ok(())
}
fn reply(code: &str) -> Json<Value> {
    Json(json!({"code":code}))
}
#[derive(Clone)]
struct App {
    root: PathBuf,
    state: PathBuf,
    launcher: PathBuf,
    token: String,
    port: u16,
    deadline: Arc<AtomicU64>,
    jobs: Arc<AtomicUsize>,
    closing: Arc<AtomicBool>,
}

fn browser(port: u16) {
    // Official Windows shell operation; no resident browser launcher dependency.
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    let url: Vec<u16> = format!("http://127.0.0.1:{port}/")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            url.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        );
    }
}
pub async fn serve(root: PathBuf, state: PathBuf, launcher: PathBuf, open: bool) -> Result<()> {
    fs::create_dir_all(&state)?;
    let port = std::env::var("ZED_ZH_SETTINGS_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(18743);
    let listener = match tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await
    {
        Ok(listener) => listener,
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
            if open {
                browser(port);
            }
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    let mut random = [0u8; 32];
    getrandom::fill(&mut random).map_err(|_| anyhow::anyhow!("Nonce initialization failed"))?;
    let app = App {
        root: root.clone(),
        state: state.clone(),
        launcher,
        token: hex::encode(random),
        port,
        deadline: Arc::new(AtomicU64::new(now() + 120)),
        jobs: Arc::new(AtomicUsize::new(0)),
        closing: Arc::new(AtomicBool::new(false)),
    };
    let api = Router::new()
        .route("/config", get(config))
        .route("/status", get(status))
        .route("/ping", post(ping))
        .route("/leave", post(leave))
        .route("/close", post(close))
        .route("/language", post(language))
        .route("/browse", post(browse))
        .route("/inspect", post(inspect))
        .route("/startup", post(startup))
        .route("/launch", post(launch))
        .route("/ai/save", post(save_ai))
        .route("/ai/models", post(models))
        .route("/ai/test", post(test))
        .route("/ai/translate", post(translate));
    let router = Router::new()
        .route("/", get(index))
        .nest("/api", api)
        .fallback_service(ServeDir::new(root.join("web/dist")))
        .layer(DefaultBodyLimit::max(16384))
        .layer(middleware::from_fn_with_state(app.clone(), guard))
        .with_state(app.clone());
    if open {
        browser(port);
    }
    let opened = now();
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let close = read(&state.join("settings-stop.json"))
                    .and_then(|v| v["time"].as_f64())
                    .is_some_and(|time| time >= opened as f64);
                let deadline = app.deadline.load(Ordering::Relaxed);
                if app.jobs.load(Ordering::Relaxed) == 0
                    && (app.closing.load(Ordering::Relaxed)
                        || close
                        || (deadline != 0 && now() >= deadline))
                {
                    break;
                }
            }
        })
        .await?;
    Ok(())
}
async fn guard(State(app): State<App>, request: Request, next: Next) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if host != format!("127.0.0.1:{}", app.port) && host != format!("localhost:{}", app.port) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if request.method() != axum::http::Method::GET {
        if request
            .headers()
            .get("X-Zed-Zh-Token")
            .and_then(|h| h.to_str().ok())
            != Some(app.token.as_str())
        {
            return StatusCode::FORBIDDEN.into_response();
        }
        if let Some(origin) = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|h| h.to_str().ok())
            && origin != format!("http://127.0.0.1:{}", app.port)
            && origin != format!("http://localhost:{}", app.port)
        {
            return StatusCode::FORBIDDEN.into_response();
        }
    }
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        "X-Content-Type-Options",
        axum::http::HeaderValue::from_static("nosniff"),
    );
    response
}
async fn index(State(app): State<App>) -> Result<Html<String>, StatusCode> {
    fs::read_to_string(app.root.join("web/dist/index.html"))
        .map(|html| Html(html.replace("__ZED_ZH_TOKEN__", &app.token)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn config(State(app): State<App>) -> Json<Value> {
    let current = settings(&app.state);
    let mut safe = json!({});
    for field in [
        "zed_exe",
        "watch_enabled",
        "startup_mode",
        "replace_shortcuts",
        "provider",
        "api_base",
        "model",
        "auto_translate",
        "ui_language",
    ] {
        safe[field] = current[field].clone();
    }
    let folder = Path::new(current["zed_exe"].as_str().unwrap_or("D:\\Zed\\Zed.exe"))
        .parent()
        .unwrap_or(Path::new("D:\\Zed"))
        .display()
        .to_string();
    Json(json!({"config":safe,"folder":folder}))
}
async fn status(State(app): State<App>) -> Json<Value> {
    Json(
        json!({"tray":read(&app.state.join("tray-status.json")).unwrap_or(json!({})),"runtime":read(&app.state.join("runtime-status.json")).unwrap_or(json!({"sessions":[]}))}),
    )
}
async fn ping(State(app): State<App>) -> Json<Value> {
    app.deadline.store(0, Ordering::Relaxed);
    reply("ready")
}
async fn leave(State(app): State<App>) -> Json<Value> {
    app.deadline.store(now() + 15, Ordering::Relaxed);
    reply("ready")
}
async fn close(State(app): State<App>) -> Json<Value> {
    app.closing.store(true, Ordering::Relaxed);
    app.deadline.store(now() + 1, Ordering::Relaxed);
    reply("closing")
}
async fn language(State(app): State<App>, Json(value): Json<Value>) -> Json<Value> {
    let Some(language) = value["language"]
        .as_str()
        .filter(|v| ["zh-CN", "zh-TW", "en"].contains(v))
    else {
        return reply("failed");
    };
    let mut current = settings(&app.state);
    current["ui_language"] = json!(language);
    if write(&app.state.join("settings.json"), &current).is_ok() {
        reply("localeSaved")
    } else {
        reply("failed")
    }
}
async fn browse(Json(value): Json<Value>) -> Json<Value> {
    let folder = value["folder"].as_str().unwrap_or("").to_owned();
    let language = value["language"].as_str().unwrap_or("zh-CN").to_owned();
    let result = tokio::task::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title(if language == "en" {
                "Select Zed installation folder"
            } else {
                "选择 Zed 安装文件夹"
            })
            .set_directory(folder)
            .pick_folder()
    })
    .await;
    match result {
        Ok(Some(folder)) => Json(json!({"code":"ready","folder":folder.display().to_string()})),
        _ => reply("cancelled"),
    }
}
fn inspect_folder(root: &Path, state: &Path, folder: &str) -> Result<(PathBuf, bool)> {
    let exe = Path::new(folder).join("Zed.exe");
    let file = fs::File::open(&exe)?;
    let map = unsafe { memmap2::MmapOptions::new().map(&file)? };
    let pe = goblin::pe::PE::parse(&map)?;
    ensure!(
        pe.header
            .optional_header
            .is_some_and(|header| header.windows_fields.subsystem == 2),
        "Not graphical Zed"
    );
    let hash = crate::catalog::hash_file(&exe)?;
    let mut adapters = read(&root.join("data/ui-adapters.json")).unwrap_or(json!({"adapters":[]}));
    if let Some(cached) = read(&state.join("verified-adapters.json"))
        && let Some(entries) = cached["adapters"].as_array()
        && let Some(known) = adapters["adapters"].as_array_mut()
    {
        known.extend(entries.clone());
    }
    let supported = adapters["adapters"].as_array().is_some_and(|entries| {
        entries
            .iter()
            .any(|entry| entry["sha256"] == hash && entry["verified"] == true)
    });
    Ok((exe, supported))
}
async fn inspect(State(app): State<App>, Json(value): Json<Value>) -> Json<Value> {
    match inspect_folder(
        &app.root,
        &app.state,
        value["folder"].as_str().unwrap_or(""),
    ) {
        Ok((_, true)) => reply("compatible"),
        Ok((_, false)) => reply("notCompatible"),
        Err(_) => reply("invalidFolder"),
    }
}
fn installer(app: &App) -> Result<()> {
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(app.root.join("runtime/install-startup.ps1"))
        .env("ZED_ZH_STATE_DIR", &app.state)
        .env("ZED_ZH_LAUNCHER", &app.launcher)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(HIDDEN)
        .status()?;
    ensure!(status.success(), "Startup configuration failed");
    Ok(())
}
async fn startup(State(app): State<App>, Json(value): Json<Value>) -> Json<Value> {
    let Ok((exe, _)) = inspect_folder(
        &app.root,
        &app.state,
        value["folder"].as_str().unwrap_or(""),
    ) else {
        return reply("invalidFolder");
    };
    let Some(mode) = value["startup_mode"]
        .as_str()
        .filter(|v| ["auto", "manual"].contains(v))
    else {
        return reply("failed");
    };
    let before = settings(&app.state);
    let mut current = before.clone();
    current["zed_exe"] = json!(exe);
    current["startup_mode"] = json!(mode);
    current["replace_shortcuts"] = json!(value["replace_shortcuts"].as_bool().unwrap_or(true));
    current["initialized"] = json!(true);
    if write(&app.state.join("settings.json"), &current).is_err() {
        return reply("failed");
    };
    if installer(&app).is_err() {
        if settings(&app.state) == current {
            let _ = write(&app.state.join("settings.json"), &before);
        }
        return reply("failed");
    };
    let _ = Command::new(&app.launcher)
        .arg("--watch")
        .env("ZED_ZH_STATE_DIR", &app.state)
        .creation_flags(HIDDEN)
        .spawn();
    reply("saved")
}
async fn launch(State(app): State<App>) -> Json<Value> {
    if Command::new(&app.launcher)
        .env("ZED_ZH_STATE_DIR", &app.state)
        .creation_flags(HIDDEN)
        .spawn()
        .is_ok()
    {
        reply("started")
    } else {
        reply("failed")
    }
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct AiInput {
    provider: String,
    api_base: String,
    model: String,
    key: String,
    store_key: bool,
    auto_translate: bool,
}
impl AiInput {
    fn validate(&self, model: bool) -> Result<()> {
        ensure!(
            ["deepseek", "glm", "zai", "compatible"].contains(&self.provider.as_str()),
            "Unknown provider"
        );
        ensure!(
            self.key.len() <= 8192 && self.model.len() <= 256 && self.api_base.len() <= 2048,
            "Invalid field size"
        );
        let config = crate::translation::Config {
            api_base: self.api_base.trim().into(),
            model: if model {
                self.model.trim().into()
            } else {
                "model-list".into()
            },
            allow_local_http: true,
            ..Default::default()
        };
        config.validate()?;
        ensure!(
            self.key.is_empty()
                || (!self.api_base.contains(self.key.trim())
                    && !self.model.contains(self.key.trim())),
            "Credential in configuration"
        );
        Ok(())
    }
    fn id(&self) -> String {
        format!(
            "{}:{}",
            self.provider,
            &hex::encode(Sha256::digest(
                self.api_base.trim().trim_end_matches('/').as_bytes()
            ))[..24]
        )
    }
    fn entry(&self) -> Result<keyring::Entry> {
        Ok(keyring::Entry::new("zed-zh-addon-ai-native", &self.id())?)
    }
    fn stored(&self) -> Result<String> {
        if let Ok(key) = self.entry()?.get_password() {
            return Ok(key);
        }
        let id = self.id();
        for target in [format!("{id}@zed-zh-addon-ai"), "zed-zh-addon-ai".into()] {
            let legacy = keyring::Entry::new_with_target(&target, "zed-zh-addon-ai", &id)?;
            if target == "zed-zh-addon-ai"
                && legacy
                    .get_attributes()
                    .ok()
                    .and_then(|v| v.get("username").cloned())
                    .as_deref()
                    != Some(id.as_str())
            {
                continue;
            }
            if let Ok(key) = legacy.get_password() {
                return Ok(key);
            }
        }
        anyhow::bail!("No saved credential")
    }
    fn actual(&self) -> Result<String> {
        let key = if self.key.trim().is_empty() {
            self.stored()
        } else {
            Ok(self.key.trim().to_owned())
        }?;
        ensure!(!key.trim().is_empty(), "Empty credential");
        Ok(key)
    }
    fn client(&self, key: &str) -> Result<Client<OpenAIConfig>> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(18))
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Client::with_config(
            OpenAIConfig::new()
                .with_api_key(key)
                .with_api_base(self.api_base.trim().trim_end_matches('/')),
        )
        .with_http_client(http))
    }
}
async fn save_ai(State(app): State<App>, Json(input): Json<AiInput>) -> Json<Value> {
    if input.model.trim().is_empty() {
        return reply("missingModel");
    };
    if input.validate(true).is_err() {
        return reply("failed");
    };
    if input.store_key
        && !input.key.trim().is_empty()
        && input
            .entry()
            .and_then(|entry| Ok(entry.set_password(input.key.trim())?))
            .is_err()
    {
        return reply("failed");
    };
    if input.auto_translate && input.stored().is_err() {
        return reply("credentialRequired");
    };
    let mut current = settings(&app.state);
    current["provider"] = json!(input.provider);
    current["api_base"] = json!(input.api_base.trim().trim_end_matches('/'));
    current["model"] = json!(input.model.trim());
    current["auto_translate"] = json!(input.auto_translate);
    if write(&app.state.join("settings.json"), &current).is_ok() {
        reply("saved")
    } else {
        reply("failed")
    }
}
async fn models(Json(input): Json<AiInput>) -> Json<Value> {
    if input.validate(false).is_err() {
        return reply("failed");
    };
    let Ok(key) = input.actual() else {
        return reply("missingKey");
    };
    let result = async {
        let client = input.client(&key)?;
        let body: Value =
            tokio::time::timeout(Duration::from_secs(20), client.models().list_byot()).await??;
        let ids = body["data"]
            .as_array()
            .context("No model list")?
            .iter()
            .filter_map(|model| model["id"].as_str())
            .filter(|id| id.len() <= 256 && !id.contains(&key))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        Ok::<_, anyhow::Error>(ids)
    }
    .await;
    match result {
        Ok(models) => Json(json!({"code":"modelsLoaded","models":models})),
        Err(_) => reply("failed"),
    }
}
async fn test(Json(input): Json<AiInput>) -> Json<Value> {
    if input.model.trim().is_empty() {
        return reply("missingModel");
    };
    if input.validate(true).is_err() {
        return reply("failed");
    };
    let Ok(key) = input.actual() else {
        return reply("missingKey");
    };
    let result=async{let client=input.client(&key)?;let mut body=json!({"model":input.model.trim(),"messages":[{"role":"user","content":"Reply only OK for a connection test."}],"max_tokens":128});if ["deepseek","glm","zai"].contains(&input.provider.as_str()){body["thinking"]=json!({"type":"disabled"});}let response:Value=tokio::time::timeout(Duration::from_secs(25),client.chat().create_byot(body)).await??;ensure!(response["choices"].as_array().is_some_and(|choices|!choices.is_empty()),"No model response");Ok::<_,anyhow::Error>(())}.await;
    if result.is_ok() {
        reply("connected")
    } else {
        reply("failed")
    }
}
async fn translation(root: &Path, state: &Path, input: &AiInput) -> Result<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("zed-zh-addon-native")
        .build()?;
    let jobs: crate::translation::Jobs = if std::env::var("ZED_ZH_OFFLINE_JOBS").as_deref()
        == Ok("1")
    {
        serde_json::from_slice(&fs::read(root.join("data/translation-jobs.json"))?)?
    } else {
        match client.get("https://raw.githubusercontent.com/dogxie2333/zed-zh-addon/main/data/translation-jobs.json").send().await{Ok(response) if response.status().is_success()=>response.json().await?,_=>serde_json::from_slice(&fs::read(root.join("data/translation-jobs.json"))?)?}
    };
    ensure!(
        jobs.schema == 1 && jobs.locale == "zh-CN",
        "Invalid public job"
    );
    if jobs.batches.is_empty() {
        return Ok(json!({"code":"nothingPending"}));
    }
    input.validate(true)?;
    let key = input.actual()?;
    let mut config = crate::translation::Config {
        api_base: input.api_base.trim().into(),
        model: input.model.trim().into(),
        allow_local_http: true,
        use_legacy_max_tokens: true,
        ..Default::default()
    };
    if ["deepseek", "glm", "zai"].contains(&input.provider.as_str()) {
        config
            .extra_body
            .insert("thinking".into(), json!({"type":"disabled"}));
    }
    let result = crate::translation::native_drafts(
        &jobs,
        &config,
        &state.join("draft-translations.redb"),
        &key,
    )
    .await?;
    fs::create_dir_all(state.join("drafts"))?;
    write(&state.join("drafts/native.json"), &result)?;
    Ok(
        json!({"code":"translated","count":result["translations"].as_object().map_or(0,|v|v.len()),"cache_hits":result["local_cache_hits"],"runtime_compatibility_approved":false}),
    )
}
async fn translate(State(app): State<App>, Json(input): Json<AiInput>) -> Json<Value> {
    if app
        .jobs
        .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return reply("busy");
    };
    let result = translation(&app.root, &app.state, &input).await;
    app.jobs.store(0, Ordering::SeqCst);
    match result {
        Ok(value) => Json(value),
        Err(_) => reply("failed"),
    }
}
pub async fn automatic(root: &Path, state: &Path) -> Result<()> {
    let cfg = settings(state);
    if cfg["auto_translate"] != true {
        return Ok(());
    };
    let input = AiInput {
        provider: cfg["provider"].as_str().unwrap_or("compatible").into(),
        api_base: cfg["api_base"].as_str().unwrap_or("").into(),
        model: cfg["model"].as_str().unwrap_or("").into(),
        ..Default::default()
    };
    let value = translation(root, state, &input).await?;
    write(&state.join("native-maintenance-status.json"), &value)
}
