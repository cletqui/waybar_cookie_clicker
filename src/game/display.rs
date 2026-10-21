use crate::format;
use crate::game::defs::{self, Effect};
use crate::game::engine::{self, SlotItem};
use crate::game::state::GameState;

/// Escape a string for embedding inside a JSON double-quoted value.
fn json_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// Waybar JSON for the main cookie button.
pub fn show(state: &GameState) -> String {
    let (cpc, cps) = engine::compute_stats(state);
    let building_defs = defs::buildings();
    let upgrade_defs = defs::upgrades();

    // ── Tooltip ───────────────────────────────────────────────────────────────
    let mut tip = format!(
        "🍪 {} cookies\n⚡ {}/sec  󰳽 {}/click",
        format::cookies(state.cookies),
        format::cookies(cps),
        format::cookies(cpc),
    );

    // Buildings section
    let owned: Vec<_> = building_defs
        .iter()
        .filter(|d| state.building_count(d.id) > 0)
        .collect();

    if !owned.is_empty() {
        tip.push_str("\n\nBuildings:");
        for def in &owned {
            let count = state.building_count(def.id);
            let mut mult = 1.0f64;
            for u in upgrade_defs
                .iter()
                .filter(|u| state.upgrades_purchased.contains(u.id))
            {
                if let Effect::BuildingMultiplier { id, factor } = &u.effect
                    && *id == def.id
                {
                    mult *= factor;
                }
            }
            let bld_cps = def.base_cps * count as f64 * mult;
            tip.push_str(&format!(
                "\n{} {} x{}  ({}/s)",
                def.icon,
                def.name,
                count,
                format::cookies(bld_cps)
            ));
        }
    }

    // Affordable upgrades are now visible in the slot panel — just flag the class
    let has_upgrade = engine::available_slots(state)
        .iter()
        .any(|s| matches!(s, SlotItem::Upgrade { .. }));

    // Main button text: icon only when 0 cookies, icon + count otherwise
    let text = if state.cookies < 1.0 {
        "󰆘".to_string()
    } else {
        format!("󰆘 {}", format::cookies(state.cookies))
    };
    let class = if has_upgrade {
        r#"["cookie-button","has-upgrade"]"#
    } else {
        r#"["cookie-button"]"#
    };

    format!(
        r#"{{"text":"{}","tooltip":"{}","class":{}}}"#,
        json_str(&text),
        json_str(&tip),
        class
    )
}

/// Waybar JSON for the Nth slot module. Empty text triggers hide-empty-text.
pub fn slot(state: &GameState, index: usize) -> String {
    let slots = engine::available_slots(state);
    let building_defs = defs::buildings();
    let upgrade_defs = defs::upgrades();

    let Some(slot) = slots.get(index) else {
        return r#"{"text":""}"#.into();
    };

    match slot {
        SlotItem::Upgrade { id, cost } => {
            let Some(def) = upgrade_defs.iter().find(|u| u.id == id) else {
                return r#"{"text":""}"#.into();
            };
            // Upgrades only appear when affordable — always show as "affordable"
            let text = format!("{} {}", def.icon, format::cookies(*cost));
            let tip = format!(
                "{}\nCost: {} 🍪\n{}",
                def.name,
                format::cookies(*cost),
                def.description
            );
            format!(
                r#"{{"text":"{}","tooltip":"{}","class":["cookie-slot","affordable"]}}"#,
                json_str(&text),
                json_str(&tip),
            )
        }
        SlotItem::Building { id, cost } => {
            let Some(def) = building_defs.iter().find(|d| d.id == id) else {
                return r#"{"text":""}"#.into();
            };
            let owned = state.building_count(id);
            let class = if *cost <= state.cookies {
                r#"["cookie-slot","affordable"]"#
            } else {
                r#"["cookie-slot","locked"]"#
            };
            let text = format!("{} {}", def.icon, format::cookies(*cost));
            let tip = format!(
                "{} (owned: {})\nCost: {} 🍪\n{}",
                def.name,
                owned,
                format::cookies(*cost),
                def.description
            );
            format!(
                r#"{{"text":"{}","tooltip":"{}","class":{}}}"#,
                json_str(&text),
                json_str(&tip),
                class
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::state::BuildingState;
    use serde_json::Value;

    fn parse(s: &str) -> Value {
        serde_json::from_str(s).expect("valid json")
    }

    fn classes(v: &Value) -> Vec<&str> {
        v["class"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect()
    }

    fn rich(cookies: f64) -> GameState {
        GameState {
            cookies,
            total_cookies: cookies,
            ..Default::default()
        }
    }

    #[test]
    fn json_str_escapes_specials() {
        assert_eq!(json_str("a\"b\\c\nd\te"), "a\\\"b\\\\c\\nd\\te");
    }

    #[test]
    fn show_with_no_cookies_is_icon_only() {
        let v = parse(&show(&GameState::default()));
        assert_eq!(v["text"], "󰆘");
        assert_eq!(classes(&v), ["cookie-button"]);
    }

    #[test]
    fn show_includes_count_and_stats_in_tooltip() {
        let v = parse(&show(&rich(1500.0)));
        assert_eq!(v["text"], "󰆘 1.5k");
        let tip = v["tooltip"].as_str().unwrap();
        assert!(tip.contains("1.5k cookies"));
        assert!(tip.contains("/sec"));
        assert!(tip.contains("/click"));
    }

    #[test]
    fn show_flags_has_upgrade_when_an_upgrade_is_affordable() {
        let v = parse(&show(&rich(100.0)));
        assert_eq!(classes(&v), ["cookie-button", "has-upgrade"]);
    }

    #[test]
    fn show_lists_owned_buildings_with_boosted_cps() {
        let mut state = GameState {
            buildings: vec![BuildingState {
                id: "cursor".into(),
                count: 10,
            }],
            ..Default::default()
        };
        state.upgrades_purchased.insert("cursor_1".into());
        let tip = parse(&show(&state))["tooltip"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(tip.contains("Cursor x10"));
        assert!(tip.contains("(2/s)"));
    }

    #[test]
    fn out_of_range_slot_is_empty() {
        let v = parse(&slot(&GameState::default(), 0));
        assert_eq!(v["text"], "");
    }

    #[test]
    fn affordable_building_slot() {
        let v = parse(&slot(&rich(15.0), 0));
        assert_eq!(classes(&v), ["cookie-slot", "affordable"]);
        assert!(v["text"].as_str().unwrap().ends_with("15"));
        assert!(v["tooltip"].as_str().unwrap().contains("Cursor (owned: 0)"));
    }

    #[test]
    fn owned_but_unaffordable_building_is_locked() {
        let state = GameState {
            buildings: vec![BuildingState {
                id: "cursor".into(),
                count: 1,
            }],
            ..Default::default()
        };
        let v = parse(&slot(&state, 0));
        assert_eq!(classes(&v), ["cookie-slot", "locked"]);
    }

    #[test]
    fn upgrade_slot_comes_first_and_is_affordable() {
        let v = parse(&slot(&rich(100.0), 0));
        assert_eq!(classes(&v), ["cookie-slot", "affordable"]);
        assert!(v["tooltip"].as_str().unwrap().contains("Cost: 100"));
    }
}
