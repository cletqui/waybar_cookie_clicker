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

    for upg in upgrades.iter().filter(|u| state.upgrades_purchased.contains(u.id)) {
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
        for upg in upgrades.iter().filter(|u| state.upgrades_purchased.contains(u.id)) {
            if let Effect::BuildingMultiplier { id, factor } = &upg.effect {
                if *id == bstate.id {
                    mult *= factor;
                }
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
        .map(|u| SlotItem::Upgrade { id: u.id.to_string(), cost: u.cost })
        .collect();
    upgrade_slots.sort_by(|a, b| a.cost().partial_cmp(&b.cost()).unwrap());

    // Buildings: only if affordable or already owned (you see what you can buy + what you have)
    let mut building_slots: Vec<SlotItem> = buildings
        .iter()
        .filter_map(|d| {
            let owned = state.building_count(d.id);
            let cost = d.next_cost(owned);
            if owned > 0 || cost <= state.cookies {
                Some(SlotItem::Building { id: d.id.to_string(), cost })
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
                    None => state.buildings.push(BuildingState { id: id.clone(), count: 1 }),
                }
            }
        }
    }
}
