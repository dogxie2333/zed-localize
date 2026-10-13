//! Process existence and translation readiness are separate states.
use serde_json::Value;

pub fn classify(
    runtime: &Value,
    config: &Value,
    failed: bool,
    now: f64,
) -> (&'static str, &'static str) {
    if failed {
        return ("监控异常：请打开配置页面", "error");
    }
    let Some(pids) = runtime["zed_pids"]
        .as_array()
        .filter(|_| now - runtime["updated"].as_f64().unwrap_or_default() < 8.0)
    else {
        return ("正在确认 Zed 状态", "checking");
    };
    let running = !pids.is_empty();
    if config["watch_enabled"] == false {
        return (
            if running {
                "Zed 已运行 · 翻译已暂停"
            } else {
                "Zed 未运行 · 翻译已暂停"
            },
            "paused",
        );
    }
    if config["translation_language"] == "en" {
        return (
            if running {
                "Zed 已运行 · 英文原文"
            } else {
                "Zed 未运行 · 英文原文"
            },
            "english",
        );
    }
    if !running {
        return ("Zed 未运行 · 补丁待命", "waiting");
    }
    let sessions = runtime["sessions"].as_array();
    let locale = config["translation_language"].as_str().unwrap_or("zh-CN");
    let ready = pids
        .iter()
        .filter(|pid| {
            sessions.is_some_and(|items| {
                items.iter().any(|item| {
                    item["pid"] == **pid
                        && ["chinese", "localized"].contains(&item["mode"].as_str().unwrap_or(""))
                        && item["locale"].as_str().unwrap_or("zh-CN") == locale
                        && now - item["updated"].as_f64().unwrap_or_default() < 8.0
                })
            })
        })
        .count();
    if ready == pids.len() {
        return ("Zed 已运行 · 翻译已加载", "active");
    }
    if ready > 0 {
        return ("Zed 已运行 · 部分窗口正在加载", "partial");
    }
    if let Some(attempts) = runtime["attachments"].as_array() {
        for item in attempts.iter().filter(|item| pids.contains(&item["pid"])) {
            if item["mode"] == "unsupported" {
                return ("Zed 已运行 · 未识别显示结构", "unsupported");
            }
            if item["mode"] == "failed" {
                return ("Zed 已运行 · 翻译加载失败", "error");
            }
        }
    }
    ("Zed 已运行 · 正在加载翻译", "loading")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn running_is_not_confused_with_missing_or_stale_engine_data() {
        let cfg = json!({"translation_language":"zh-CN"});
        let mut runtime = json!({"updated":100.0,"zed_pids":[11],"sessions":[]});
        assert_eq!(classify(&runtime, &cfg, false, 101.0).1, "loading");
        assert_eq!(classify(&runtime, &cfg, false, 110.0).1, "checking");
        runtime["sessions"] = json!([{"pid":22,"mode":"chinese","updated":100.0}]);
        assert_eq!(classify(&runtime, &cfg, false, 101.0).1, "loading");
        runtime["sessions"] = json!([{"pid":11,"mode":"chinese","updated":100.0,"locale":"zh-CN"}]);
        assert_eq!(classify(&runtime, &cfg, false, 101.0).1, "active");
        runtime["zed_pids"] = json!([]);
        assert_eq!(classify(&runtime, &cfg, false, 101.0).1, "waiting");
    }
    #[test]
    fn unsupported_and_failed_processes_are_visible() {
        let cfg = json!({});
        let mut runtime = json!({"updated":100.0,"zed_pids":[11],"sessions":[],"attachments":[{"pid":11,"mode":"unsupported"}]});
        assert_eq!(classify(&runtime, &cfg, false, 101.0).1, "unsupported");
        runtime["attachments"][0]["mode"] = json!("failed");
        assert_eq!(classify(&runtime, &cfg, false, 101.0).1, "error");
    }
}
