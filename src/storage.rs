use crate::game::state::GameState;
use std::path::{Path, PathBuf};

pub fn default_state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let dir = PathBuf::from(home).join(".local/share/waybar_cookie_clicker");
    std::fs::create_dir_all(&dir).ok();
    dir.join("state.json")
}

pub fn load(path: &Path) -> GameState {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(path: &Path, state: &GameState) {
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            std::fs::rename(&tmp, path).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_loads_default() {
        let dir = tempfile::tempdir().unwrap();
        let state = load(&dir.path().join("none.json"));
        assert_eq!(state.cookies, 0.0);
    }

    #[test]
    fn corrupt_file_loads_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(load(&path).cookies, 0.0);
    }

    #[test]
    fn save_then_load_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        let mut state = GameState {
            cookies: 42.5,
            total_cookies: 100.0,
            last_tick_ms: 9,
            ..Default::default()
        };
        state.upgrades_purchased.insert("iron_mouse".into());
        save(&path, &state);
        let back = load(&path);
        assert_eq!(back.cookies, 42.5);
        assert_eq!(back.total_cookies, 100.0);
        assert_eq!(back.last_tick_ms, 9);
        assert!(back.upgrades_purchased.contains("iron_mouse"));
        assert!(!path.with_extension("json.tmp").exists());
    }
}
