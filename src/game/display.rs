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
    let class = if has_upgrade { "has-upgrade" } else { "" };

    format!(
        r#"{{"text":"{}","tooltip":"{}","class":"{}"}}"#,
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
                r#"{{"text":"{}","tooltip":"{}","class":"affordable"}}"#,
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
                "affordable"
            } else {
                "locked"
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
                r#"{{"text":"{}","tooltip":"{}","class":"{}"}}"#,
                json_str(&text),
                json_str(&tip),
                class
            )
        }
    }
}
