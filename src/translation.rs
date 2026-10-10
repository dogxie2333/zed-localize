use anyhow::{Context, Result, ensure};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ResponseFormat, ResponseFormatJsonSchema,
    },
};
use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

const TABLE: TableDefinition<&str, &str> = TableDefinition::new("validated_translation_v1");
const POLICY: &str = "zed-zh-semantic-context-v1";

#[derive(Clone, Deserialize, Serialize)]
pub struct Jobs {
    pub schema: u32,
    pub locale: String,
    pub upstream_commit: String,
    pub source_commit: String,
    pub validator_fingerprint: String,
    pub review_required: Vec<String>,
    pub system_prompt: String,
    pub batches: Vec<Batch>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Batch {
    pub entries: Vec<Value>,
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub api_base: String,
    pub model: String,
    pub api_key_env: String,
    pub timeout_seconds: u64,
    pub max_batches: usize,
    pub max_output_tokens: u32,
    pub strict_schema: bool,
    pub provider_prompt_cache_key: bool,
    pub allow_local_http: bool,
    pub use_legacy_max_tokens: bool,
    pub json_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_base: "https://api.openai.com/v1".into(),
            model: String::new(),
            api_key_env: "ZED_I18N_API_KEY".into(),
            timeout_seconds: 90,
            max_batches: 20,
            max_output_tokens: 4096,
            strict_schema: false,
            provider_prompt_cache_key: false,
            allow_local_http: false,
            use_legacy_max_tokens: false,
            json_mode: true,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.model.trim().is_empty(), "Configure a provider model");
        ensure!(
            self.timeout_seconds > 0
                && self.timeout_seconds <= 600
                && self.max_batches > 0
                && self.max_output_tokens > 0,
            "Invalid maintenance limits"
        );
        let url = reqwest::Url::parse(&self.api_base)
            .map_err(|_| anyhow::anyhow!("Invalid API base URL"))?;
        ensure!(
            url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "API base must not contain credentials, query parameters or fragments"
        );
        let loopback = url.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
        ensure!(
            url.scheme() == "https"
                || (self.allow_local_http && url.scheme() == "http" && loopback),
            "Use HTTPS; local HTTP requires explicit test opt-in"
        );
        Ok(())
    }
}

fn stable_context(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, _)| {
                    !matches!(
                        key.as_str(),
                        "line"
                            | "start_line"
                            | "end_line"
                            | "start_byte"
                            | "end_byte"
                            | "batch_index"
                            | "batch_count"
                            | "from_version"
                            | "to_version"
                    ) && !(key.as_str() == "id"
                        && object.contains_key("start_line")
                        && object.contains_key("type"))
                        && !(key.as_str() == "target"
                            && object.contains_key("source")
                            && object.contains_key("role"))
                })
                .map(|(key, value)| (key.clone(), stable_context(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(stable_context).collect()),
        _ => value.clone(),
    }
}

pub fn semantic_entry(entry: &Value) -> Result<Value> {
    ensure!(
        entry["source"].is_string(),
        "Translation entry has no source"
    );
    let mut value = stable_context(entry);
    if let Some(context) = value["code_context"].as_str() {
        let lines = context
            .lines()
            .map(|line| {
                if let Some((prefix, text)) = line.split_once(':')
                    && prefix.trim().parse::<usize>().is_ok()
                {
                    return text;
                }
                line
            })
            .collect::<Vec<_>>()
            .join("\n");
        value["code_context"] = Value::String(lines);
    }
    if let Some(occurrences) = value["occurrences"].as_array() {
        let unique = occurrences
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<BTreeSet<_>, _>>()?;
        value["occurrences"] = Value::Array(
            unique
                .iter()
                .map(|text| serde_json::from_str(text))
                .collect::<std::result::Result<_, _>>()?,
        );
    }
    Ok(value)
}

pub fn cache_key(jobs: &Jobs, config: &Config, entry: &Value) -> Result<String> {
    let payload = json!({"policy":POLICY,"locale":jobs.locale,"model":config.model,
        "provider":config.api_base.trim_end_matches('/'),"prompt":jobs.system_prompt,
        "validator":jobs.validator_fingerprint,"entry":semantic_entry(entry)?});
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(&payload)?)))
}

struct Cache {
    database: Database,
}
impl Cache {
    fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let database = Database::create(path)?;
        let write = database.begin_write()?;
        {
            write.open_table(TABLE)?;
        }
        write.commit()?;
        Ok(Self { database })
    }
    fn get(&self, key: &str) -> Result<Option<Value>> {
        let read = self.database.begin_read()?;
        let table = read.open_table(TABLE)?;
        let value = table.get(key)?.map(|value| value.value().to_owned());
        value
            .map(|text| serde_json::from_str(&text).map_err(Into::into))
            .transpose()
    }
    fn put(&self, values: &[(String, Value)]) -> Result<()> {
        let write = self.database.begin_write()?;
        {
            let mut table = write.open_table(TABLE)?;
            for (key, value) in values {
                table.insert(key.as_str(), serde_json::to_string(value)?.as_str())?;
            }
        }
        write.commit()?;
        Ok(())
    }
}

pub trait Validator {
    fn validate(&self, jobs: &Jobs, batch: &Batch, response: &Value) -> Result<()>;
}

struct CommunityValidator<'a> {
    toolkit: &'a Path,
    python: &'a Path,
    bridge: &'a Path,
    key_env: &'a str,
}
impl Validator for CommunityValidator<'_> {
    fn validate(&self, jobs: &Jobs, batch: &Batch, response: &Value) -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let batch_file = temporary.path().join("batch.json");
        let response_file = temporary.path().join("response.json");
        fs::write(&batch_file, serde_json::to_vec(batch)?)?;
        fs::write(&response_file, serde_json::to_vec(response)?)?;
        let status = Command::new(self.python)
            .arg(self.bridge)
            .arg("validate-response")
            .arg("--toolkit")
            .arg(self.toolkit)
            .arg("--batch")
            .arg(&batch_file)
            .arg("--response")
            .arg(&response_file)
            .arg("--expected-toolkit-commit")
            .arg(&jobs.upstream_commit)
            .arg("--expected-validator-fingerprint")
            .arg(&jobs.validator_fingerprint)
            .env_remove(self.key_env)
            .env_remove("OPENAI_API_KEY")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        ensure!(
            status.success(),
            "Response rejected by the community validator; nothing from this batch was cached"
        );
        Ok(())
    }
}

pub async fn translate_jobs(
    jobs: &Jobs,
    config: &Config,
    cache_path: &Path,
    refresh: bool,
    validator: &dyn Validator,
    api_key: impl Fn() -> Result<String>,
) -> Result<Value> {
    ensure!(
        jobs.schema == 1 && jobs.locale == "zh-CN",
        "Unsupported translation job"
    );
    config.validate()?;
    let cache = Cache::open(cache_path)?;
    let mut output = BTreeMap::<String, Value>::new();
    let mut hits = 0usize;
    let mut requests = 0usize;
    let mut provider_cached_tokens = 0u64;
    let mut client = None;
    let mut secret = None;
    let prefix_key = format!(
        "zed-zh:{}",
        &hex::encode(Sha256::digest(jobs.system_prompt.as_bytes()))[..48]
    );
    for original_batch in &jobs.batches {
        let mut pending = Vec::new();
        let mut keys = Vec::new();
        for entry in &original_batch.entries {
            let source = entry["source"]
                .as_str()
                .context("Missing translation source")?;
            let key = cache_key(jobs, config, entry)?;
            if !refresh && let Some(translation) = cache.get(&key)? {
                output.insert(source.into(), translation);
                hits += 1;
                continue;
            }
            pending.push(entry.clone());
            keys.push(key);
        }
        if pending.is_empty() {
            continue;
        }
        ensure!(
            requests < config.max_batches,
            "Maintenance batch limit reached; validated batches were cached for the next run"
        );
        if client.is_none() {
            let key = api_key()?;
            ensure!(
                !key.trim().is_empty(),
                "Set the API key environment variable; do not put keys in configuration files"
            );
            ensure!(
                !config.model.contains(&key) && !config.api_base.contains(&key),
                "API credentials must not appear in model or endpoint configuration"
            );
            let http = reqwest::Client::builder()
                .timeout(Duration::from_secs(config.timeout_seconds))
                .connect_timeout(Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none())
                .build()?;
            client = Some(
                Client::with_config(
                    OpenAIConfig::new()
                        .with_api_key(&key)
                        .with_api_base(config.api_base.trim_end_matches('/')),
                )
                .with_http_client(http),
            );
            secret = Some(key);
        }
        let batch = Batch { entries: pending };
        let semantic = batch
            .entries
            .iter()
            .map(semantic_entry)
            .collect::<Result<Vec<_>>>()?;
        let system = ChatCompletionRequestSystemMessageArgs::default()
            .content(jobs.system_prompt.clone())
            .build()?;
        let user = ChatCompletionRequestUserMessageArgs::default()
            .content(serde_json::to_string(&json!({"entries":semantic}))?)
            .build()?;
        let mut builder = CreateChatCompletionRequestArgs::default();
        builder
            .model(&config.model)
            .messages(vec![system.into(), user.into()]);
        if config.use_legacy_max_tokens {
            #[allow(deprecated)]
            builder.max_tokens(config.max_output_tokens);
        } else {
            builder.max_completion_tokens(config.max_output_tokens);
        }
        if config.strict_schema {
            let properties = batch
                .entries
                .iter()
                .map(|entry| {
                    (
                        entry["source"].as_str().unwrap_or_default().to_string(),
                        json!({"type":["string","null"]}),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            let required = properties.keys().cloned().collect::<Vec<_>>();
            builder.response_format(ResponseFormat::JsonSchema { json_schema: ResponseFormatJsonSchema {
                name: "zed_translation".into(), description: None, strict: Some(true),
                schema: json!({"type":"object","properties":properties,"required":required,"additionalProperties":false}) } });
        } else if config.json_mode {
            builder.response_format(ResponseFormat::JsonObject);
        }
        if config.provider_prompt_cache_key {
            builder.prompt_cache_key(&prefix_key);
        }
        let request = builder.build()?;
        // The SDK supplies bounded transient retries. This outer deadline also
        // bounds the total batch wall time across any SDK retry/backoff.
        let response = tokio::time::timeout(
            Duration::from_secs(config.timeout_seconds),
            client
                .as_ref()
                .context("Missing API client")?
                .chat()
                .create(request),
        )
        .await
        .map_err(|_| anyhow::anyhow!("AI batch deadline exceeded"))?
        .map_err(|_| {
            anyhow::anyhow!(
                "AI request failed; provider response details are intentionally not logged"
            )
        })?;
        requests += 1;
        if let Some(usage) = &response.usage {
            provider_cached_tokens += usage
                .prompt_tokens_details
                .as_ref()
                .and_then(|details| details.cached_tokens)
                .unwrap_or(0) as u64;
        }
        let choice = response
            .choices
            .first()
            .context("Provider returned no choice")?;
        let content = choice
            .message
            .content
            .as_deref()
            .context("Provider returned no translation JSON")?;
        ensure!(
            !content.contains(secret.as_deref().context("Missing API key")?),
            "Provider response contains sensitive data; nothing was persisted"
        );
        let translated: Value = serde_json::from_str(content).map_err(|_| {
            anyhow::anyhow!(
                "Provider did not return valid JSON; nothing from this batch was cached"
            )
        })?;
        let object = translated
            .as_object()
            .context("Provider returned a non-object translation")?;
        let sources = batch
            .entries
            .iter()
            .map(|entry| entry["source"].as_str().unwrap_or_default())
            .collect::<BTreeSet<_>>();
        ensure!(
            object.keys().map(String::as_str).collect::<BTreeSet<_>>() == sources
                && object
                    .values()
                    .all(|value| value.is_string() || value.is_null()),
            "Provider returned missing/extra keys or invalid values"
        );
        validator.validate(jobs, &batch, &translated)?;
        let mut validated = Vec::new();
        for (entry, key) in batch.entries.iter().zip(keys) {
            let source = entry["source"].as_str().context("Missing source")?;
            let value = object[source].clone();
            validated.push((key, value.clone()));
            output.insert(source.into(), value);
        }
        cache.put(&validated)?;
    }
    Ok(
        json!({"schema":1,"status":"draft","source_commit":jobs.source_commit,"upstream_commit":jobs.upstream_commit,
        "locale":jobs.locale,"model":config.model,"translations":output,"local_cache_hits":hits,"batch_requests":requests,
        "provider_cached_tokens":provider_cached_tokens,"review_required":jobs.review_required,
        "unresolved_count":output.values().filter(|value| value.is_null()).count(),"runtime_compatibility_approved":false}),
    )
}

pub async fn translate(
    jobs_file: &Path,
    config_file: &Path,
    cache_path: &Path,
    toolkit: &Path,
    python: &Path,
    bridge: &Path,
    refresh: bool,
) -> Result<Value> {
    let jobs: Jobs = serde_json::from_slice(&fs::read(jobs_file)?)?;
    let config: Config = toml::from_str(&fs::read_to_string(config_file)?).map_err(|_| {
        anyhow::anyhow!("Invalid translation configuration (contents are not logged)")
    })?;
    let validator = CommunityValidator {
        toolkit,
        python,
        bridge,
        key_env: &config.api_key_env,
    };
    translate_jobs(&jobs, &config, cache_path, refresh, &validator, || {
        std::env::var(&config.api_key_env)
            .map_err(|_| anyhow::anyhow!("API key environment variable is not set"))
    })
    .await
}

pub async fn check_update() -> Result<Value> {
    let current = crate::catalog::embedded_catalog()?.source_version;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("zed-zh-addon/0.1.0")
        .build()?;
    let latest: Value = client
        .get("https://api.github.com/repos/zed-industries/zed/releases/latest")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let tag = latest["tag_name"]
        .as_str()
        .context("No upstream release tag")?;
    Ok(
        json!({"current_source_version":current,"latest_tag":tag,"update_available":tag.trim_start_matches('v')!=current,
        "installed_program_changed":false,"runtime_compatibility_approved":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture_jobs() -> Jobs {
        Jobs {
            schema: 1,
            locale: "zh-CN".into(),
            upstream_commit: "public-upstream".into(),
            source_commit: "old".into(),
            validator_fingerprint: "validator-v1".into(),
            review_required: vec![],
            system_prompt: "Stable glossary and code-editor background".into(),
            batches: vec![],
        }
    }
    fn config() -> Config {
        Config {
            model: "mock-model".into(),
            ..Config::default()
        }
    }
    #[test]
    fn semantic_cache_survives_line_and_version_changes() {
        let mut jobs = fixture_jobs();
        let a = json!({"source":"Open Settings","kind":"menu_item","code_context":"81: MenuItem::action(\"Open Settings\", OpenSettings)",
            "occurrences":[{"file":"crates/zed/menu.rs","line":81,"start_byte":100,"end_byte":120,"call":"MenuItem::action"}]});
        let mut b = a.clone();
        b["code_context"] = json!("91: MenuItem::action(\"Open Settings\", OpenSettings)");
        b["occurrences"][0]["line"] = json!(91);
        b["occurrences"][0]["start_byte"] = json!(200);
        let key = cache_key(&jobs, &config(), &a).unwrap();
        jobs.source_commit = "new-version".into();
        assert_eq!(key, cache_key(&jobs, &config(), &b).unwrap());
        b["code_context"] = json!("91: MenuItem::action(\"Open Settings\", OpenProjectSettings)");
        assert_ne!(key, cache_key(&jobs, &config(), &b).unwrap());
    }
    #[test]
    fn glossary_model_and_validator_changes_invalidate_cache() {
        let mut jobs = fixture_jobs();
        let entry = json!({"source":"Thread","kind":"label"});
        let key = cache_key(&jobs, &config(), &entry).unwrap();
        jobs.system_prompt
            .push_str(" Thread means AI conversation, not OS thread");
        assert_ne!(key, cache_key(&jobs, &config(), &entry).unwrap());
        let mut cfg = config();
        cfg.model = "other-model".into();
        assert_ne!(key, cache_key(&fixture_jobs(), &cfg, &entry).unwrap());
    }

    #[test]
    fn group_line_ids_and_batch_target_flags_are_not_semantic() {
        let jobs = fixture_jobs();
        let a = json!({"source":"Open Settings","kind":"setting_title","context_group":{
            "id":"setting:crates/settings/page.rs:81","type":"setting","file":"crates/settings/page.rs",
            "start_line":81,"end_line":90,"context_key":"ui.settings","entries":[
                {"source":"Open Settings","role":"title","target":true,"line":81},
                {"source":"Choose editor preferences","role":"description","target":false,"line":83}]}});
        let mut b = a.clone();
        b["context_group"]["id"] = json!("setting:crates/settings/page.rs:91");
        b["context_group"]["start_line"] = json!(91);
        b["context_group"]["entries"][1]["target"] = json!(true);
        b["context_group"]["entries"][1]["line"] = json!(93);
        assert_eq!(
            cache_key(&jobs, &config(), &a).unwrap(),
            cache_key(&jobs, &config(), &b).unwrap()
        );
        b["context_group"]["entries"][1]["source"] = json!("Choose workspace preferences");
        assert_ne!(
            cache_key(&jobs, &config(), &a).unwrap(),
            cache_key(&jobs, &config(), &b).unwrap()
        );
    }
    #[test]
    fn refuse_embedded_keys_and_nonlocal_insecure_endpoints() {
        let mut cfg = config();
        cfg.api_base = "https://user:credential@example.com/v1".into();
        assert!(cfg.validate().is_err());
        cfg.api_base = "http://example.com/v1".into();
        cfg.allow_local_http = true;
        assert!(cfg.validate().is_err());
        cfg.api_base = "http://127.0.0.1:9000/v1".into();
        assert!(cfg.validate().is_ok());
    }
}
