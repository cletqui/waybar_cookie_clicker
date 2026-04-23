use std::time::{SystemTime, UNIX_EPOCH};

use crate::game::defs::{self, Effect, UnlockCondition};
use crate::game::state::{BuildingState, GameState};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Apply passive income since the last tick, then update the timestamp.
pub fn tick(state: &mut GameState) {
    let now = now_ms();
    if state.last_tick_ms > 0 {
        let elapsed = (now.saturating_sub(state.last_tick_ms)) as f64 / 1_000.0;
        // Cap offline gain at 8 hours so a long suspension doesn't break the game
        let elapsed = elapsed.min(28_800.0);
        let (_, cps) = compute_stats(state);
        let gained = cps * elapsed;
        state.cookies += gained;
        state.total_cookies += gained;
    }
    state.last_tick_ms = now;
}

pub fn click(state: &mut GameState) {
    let (cpc, _) = compute_stats(state);
    state.cookies += cpc;
    state.total_cookies += cpc;
}

/// Returns (cookies_per_click, cookies_per_second) derived from current buildings + upgrades.
pub fn compute_stats(state: &GameState) -> (f64, f64) {
    let buildings = defs::buildings();
    let upgrades = defs::upgrades();

    let mut cpc = 1.0f64;
    let mut cpc_mult = 1.0f64;
    let mut cps = 0.0f64;
    let mut global_cps_mult = 1.0f64;

    for upg in upgrades
        .iter()
        .filter(|u| state.upgrades_purchased.contains(u.id))
    {
        match &upg.effect {
            Effect::ClickBonus(n) => cpc += n,
            Effect::ClickMultiplier(f) => cpc_mult *= f,
            Effect::AllCpsMultiplier(f) => global_cps_mult *= f,
            Effect::BuildingMultiplier { .. } => {}
        }
    }
    cpc *= cpc_mult;

    for bstate in &state.buildings {
        if bstate.count == 0 {
            continue;
        }
        let Some(def) = buildings.iter().find(|d| d.id == bstate.id) else {
            continue;
        };
        let mut mult = 1.0f64;
        for upg in upgrades
            .iter()
            .filter(|u| state.upgrades_purchased.contains(u.id))
        {
            if let Effect::BuildingMultiplier { id, factor } = &upg.effect
                && *id == bstate.id
            {
                mult *= factor;
            }
        }
        cps += def.base_cps * bstate.count as f64 * mult;
    }

    cps *= global_cps_mult;
    (cpc, cps)
}

// ── Slot system ───────────────────────────────────────────────────────────────

pub enum SlotItem {
    Upgrade { id: String, cost: f64 },
    Building { id: String, cost: f64 },
}

impl SlotItem {
    pub fn cost(&self) -> f64 {
        match self {
            SlotItem::Upgrade { cost, .. } | SlotItem::Building { cost, .. } => *cost,
        }
    }
}

/// Ordered list of what appears in the expandable group.
/// Nothing appears until you can afford it or already own it — full suspense mode.
///   1. Affordable unlocked upgrades (by cost asc)
///   2. Affordable buildings + buildings you already own (by cost asc)
pub fn available_slots(state: &GameState) -> Vec<SlotItem> {
    let buildings = defs::buildings();
    let upgrades = defs::upgrades();

    let click_upgrades_bought: u32 = upgrades
        .iter()
        .filter(|u| state.upgrades_purchased.contains(u.id))
        .filter(|u| matches!(u.effect, Effect::ClickBonus(_) | Effect::ClickMultiplier(_)))
        .count() as u32;

    // Upgrades: only unlocked AND affordable
    let mut upgrade_slots: Vec<SlotItem> = upgrades
        .iter()
        .filter(|u| !state.upgrades_purchased.contains(u.id))
        .filter(|u| is_unlocked(u, state, click_upgrades_bought))
        .filter(|u| u.cost <= state.cookies)
        .map(|u| SlotItem::Upgrade {
            id: u.id.to_string(),
            cost: u.cost,
        })
        .collect();
    upgrade_slots.sort_by(|a, b| a.cost().partial_cmp(&b.cost()).unwrap());

    // Buildings: only if affordable or already owned (you see what you can buy + what you have)
    let mut building_slots: Vec<SlotItem> = buildings
        .iter()
        .filter_map(|d| {
            let owned = state.building_count(d.id);
            let cost = d.next_cost(owned);
            if owned > 0 || cost <= state.cookies {
                Some(SlotItem::Building {
                    id: d.id.to_string(),
                    cost,
                })
            } else {
                None
            }
        })
        .collect();
    building_slots.sort_by(|a, b| a.cost().partial_cmp(&b.cost()).unwrap());

    upgrade_slots.into_iter().chain(building_slots).collect()
}

fn is_unlocked(upg: &defs::UpgradeDef, state: &GameState, click_upgrades_bought: u32) -> bool {
    match &upg.unlock {
        UnlockCondition::Always => true,
        UnlockCondition::OwnBuilding { id, min } => state.building_count(id) >= *min,
        UnlockCondition::TotalCookies(n) => state.total_cookies >= *n,
        UnlockCondition::ClickUpgradeCount(n) => click_upgrades_bought >= *n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::state::BuildingState;

    fn state_with_cookies(n: f64) -> GameState {
        GameState {
            cookies: n,
            total_cookies: n,
            ..Default::default()
        }
    }

    fn state_with_building(id: &str, count: u32) -> GameState {
        GameState {
            buildings: vec![BuildingState {
                id: id.into(),
                count,
            }],
            ..Default::default()
        }
    }

    // ── compute_stats ──────────────────────────────────────────────────────────

    #[test]
    fn base_stats_no_buildings_no_upgrades() {
        let state = GameState::default();
        let (cpc, cps) = compute_stats(&state);
        assert_eq!(cpc, 1.0);
        assert_eq!(cps, 0.0);
    }

    #[test]
    fn one_cursor_produces_correct_cps() {
        let state = state_with_building("cursor", 1);
        let (_, cps) = compute_stats(&state);
        assert!((cps - 0.1).abs() < 1e-9);
    }

    #[test]
    fn multiple_cursors_stack_linearly() {
        let state = state_with_building("cursor", 10);
        let (_, cps) = compute_stats(&state);
        assert!((cps - 1.0).abs() < 1e-9);
    }

    #[test]
    fn click_bonus_upgrade_adds_to_cpc() {
        let mut state = GameState::default();
        state
            .upgrades_purchased
            .insert("reinforced_clicking".into());
        let (cpc, _) = compute_stats(&state);
        assert_eq!(cpc, 2.0);
    }

    #[test]
    fn click_multiplier_upgrade_doubles_cpc() {
        let mut state = GameState::default();
        state.upgrades_purchased.insert("iron_mouse".into());
        let (cpc, _) = compute_stats(&state);
        assert_eq!(cpc, 2.0);
    }

    #[test]
    fn building_multiplier_doubles_cps() {
        let mut state = state_with_building("cursor", 1);
        state.upgrades_purchased.insert("cursor_1".into());
        let (_, cps) = compute_stats(&state);
        assert!((cps - 0.2).abs() < 1e-9);
    }

    #[test]
    fn global_cps_multiplier_applies_to_all() {
        let mut state = state_with_building("cursor", 10); // 1.0 cps
        state.upgrades_purchased.insert("global_1".into());
        let (_, cps) = compute_stats(&state);
        assert!((cps - 1.1).abs() < 1e-9);
    }

    // ── click ──────────────────────────────────────────────────────────────────

    #[test]
    fn click_adds_cpc_to_cookies() {
        let mut state = GameState::default(); // cpc = 1.0
        click(&mut state);
        assert_eq!(state.cookies, 1.0);
        assert_eq!(state.total_cookies, 1.0);
    }

    #[test]
    fn click_increments_total_cookies() {
        let mut state = state_with_cookies(50.0);
        click(&mut state);
        assert_eq!(state.total_cookies, 51.0);
    }

    // ── tick ───────────────────────────────────────────────────────────────────

    #[test]
    fn tick_skips_income_when_last_tick_is_zero() {
        let mut state = state_with_building("cursor", 1);
        state.last_tick_ms = 0;
        tick(&mut state);
        assert_eq!(state.cookies, 0.0);
    }

    #[test]
    fn tick_applies_passive_income() {
        let mut state = state_with_building("cursor", 10); // 1.0 cps
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        state.last_tick_ms = now - 2_000; // 2 seconds ago
        tick(&mut state);
        assert!(state.cookies >= 1.9 && state.cookies <= 2.1);
    }

    #[test]
    fn tick_caps_offline_gain_at_8_hours() {
        let mut state = state_with_building("cursor", 10); // 1.0 cps
        state.last_tick_ms = 1; // far in the past → many hours
        tick(&mut state);
        let max_gain = 1.0 * 28_800.0;
        assert!(state.cookies <= max_gain + 0.001);
    }

    // ── available_slots ────────────────────────────────────────────────────────

    #[test]
    fn no_slots_with_zero_cookies() {
        let state = GameState::default();
        assert!(available_slots(&state).is_empty());
    }

    #[test]
    fn cursor_appears_when_affordable() {
        let state = state_with_cookies(15.0);
        let slots = available_slots(&state);
        assert!(
            slots
                .iter()
                .any(|s| matches!(s, SlotItem::Building { id, .. } if id == "cursor"))
        );
    }

    #[test]
    fn owned_building_always_visible() {
        let state = state_with_building("cursor", 1); // 0 cookies but owns a cursor
        let slots = available_slots(&state);
        assert!(
            slots
                .iter()
                .any(|s| matches!(s, SlotItem::Building { id, .. } if id == "cursor"))
        );
    }

    #[test]
    fn upgrade_only_visible_when_affordable() {
        // reinforced_clicking costs 100 — should not appear with 50 cookies
        let state = state_with_cookies(50.0);
        let slots = available_slots(&state);
        assert!(
            !slots
                .iter()
                .any(|s| matches!(s, SlotItem::Upgrade { id, .. } if id == "reinforced_clicking"))
        );

        // should appear with 100 cookies
        let state = state_with_cookies(100.0);
        let slots = available_slots(&state);
        assert!(
            slots
                .iter()
                .any(|s| matches!(s, SlotItem::Upgrade { id, .. } if id == "reinforced_clicking"))
        );
    }

    #[test]
    fn locked_upgrade_never_appears_even_if_affordable() {
        // mouse_wheel requires 1 click upgrade already bought
        let state = state_with_cookies(500.0); // can afford mouse_wheel (500 cost) but no click upg bought
        let slots = available_slots(&state);
        assert!(
            !slots
                .iter()
                .any(|s| matches!(s, SlotItem::Upgrade { id, .. } if id == "mouse_wheel"))
        );
    }

    // ── buy_slot ───────────────────────────────────────────────────────────────

    #[test]
    fn buy_slot_deducts_cookies() {
        let mut state = state_with_cookies(15.0); // cursor costs 15
        buy_slot(&mut state, 0);
        assert!(state.cookies < 15.0);
    }

    #[test]
    fn buy_slot_adds_building() {
        let mut state = state_with_cookies(15.0);
        buy_slot(&mut state, 0);
        assert_eq!(state.building_count("cursor"), 1);
    }

    #[test]
    fn buy_slot_does_nothing_when_unaffordable() {
        let mut state = GameState::default(); // 0 cookies, no slots
        buy_slot(&mut state, 0);
        assert_eq!(state.cookies, 0.0);
    }

    #[test]
    fn buy_slot_upgrade_added_to_purchased() {
        let mut state = state_with_cookies(100.0); // reinforced_clicking costs 100
        buy_slot(&mut state, 0); // slot 0 = reinforced_clicking (cheapest available)
        assert!(state.upgrades_purchased.contains("reinforced_clicking"));
    }
}

pub fn buy_slot(state: &mut GameState, index: usize) {
    let slots = available_slots(state);
    let Some(slot) = slots.get(index) else {
        return;
    };
    match slot {
        SlotItem::Upgrade { id, cost } => {
            if state.cookies >= *cost {
                state.cookies -= cost;
                state.upgrades_purchased.insert(id.clone());
            }
        }
        SlotItem::Building { id, cost } => {
            if state.cookies >= *cost {
                state.cookies -= cost;
                match state.buildings.iter_mut().find(|b| &b.id == id) {
                    Some(b) => b.count += 1,
                    None => state.buildings.push(BuildingState {
                        id: id.clone(),
                        count: 1,
                    }),
                }
            }
        }
    }
}
