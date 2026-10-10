use std::path::{Path, PathBuf};
use std::sync::OnceLock;
pub static STATE: OnceLock<PathBuf> = OnceLock::new();
pub fn state(root: &Path) -> PathBuf {
    STATE
        .get()
        .cloned()
        .or_else(|| std::env::var_os("ZED_ZH_STATE_DIR").map(PathBuf::from))
        .unwrap_or_else(|| root.join("local"))
}
