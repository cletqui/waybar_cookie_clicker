use crate::game::state::GameState;
use std::path::PathBuf;

pub fn default_state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let dir = PathBuf::from(home).join(".local/share/waybar_cookie_clicker");
    std::fs::create_dir_all(&dir).ok();
    dir.join("state.json")
}

pub fn load(path: &PathBuf) -> GameState {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(path: &PathBuf, state: &GameState) {
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            std::fs::rename(&tmp, path).ok();
        }
    }
}
