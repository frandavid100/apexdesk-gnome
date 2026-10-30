//! Host preferences, separate from the pinned WordCraft engine.
use std::path::PathBuf;
pub fn config_root() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"))
        .join("wordcraft-gnome")
}
pub fn recovery_root() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state"))
        .join("wordcraft-gnome/recovery")
}
/// 0 disabled, 1 recovery (default), 2 document file.
pub fn autosave_mode() -> u32 {
    std::fs::read(config_root().join("preferences.json")).ok()
        .and_then(|v| serde_json::from_slice::<serde_json::Value>(&v).ok())
        .and_then(|v| v["autosave"].as_u64()).filter(|v| *v <= 2).unwrap_or(1) as u32
}
pub fn set_autosave_mode(mode: u32) -> Result<(), String> {
    if mode > 2 { return Err("Modo de autoguardado desconocido".into()); }
    let dir = config_root(); std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let temp = dir.join(format!("preferences-{}.tmp", std::process::id()));
    std::fs::write(&temp, serde_json::json!({"autosave":mode}).to_string()).map_err(|e| e.to_string())?;
    std::fs::rename(temp, dir.join("preferences.json")).map_err(|e| e.to_string())
}
