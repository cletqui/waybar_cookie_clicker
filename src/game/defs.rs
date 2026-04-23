pub struct BuildingDef {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub base_cost: f64,
    pub base_cps: f64,
    pub description: &'static str,
}

impl BuildingDef {
    /// Cost of the next purchase given `owned` already bought (1.15x scaling)
    pub fn next_cost(&self, owned: u32) -> f64 {
        self.base_cost * 1.15f64.powi(owned as i32)
    }
}

pub fn buildings() -> Vec<BuildingDef> {
    vec![
        BuildingDef {
            id: "cursor",
            name: "Cursor",
            icon: "󰳽",
            base_cost: 15.0,
            base_cps: 0.1,
            description: "Autoclicks the big cookie.",
        },
        BuildingDef {
            id: "grandma",
            name: "Grandma",
            icon: "󰜥",
            base_cost: 100.0,
            base_cps: 0.5,
            description: "A nice grandma to bake more cookies.",
        },
        BuildingDef {
            id: "farm",
            name: "Farm",
            icon: "󱢲",
            base_cost: 1_100.0,
            base_cps: 4.0,
            description: "Grows cookie plants from cookie seeds.",
        },
        BuildingDef {
            id: "mine",
            name: "Mine",
            icon: "󱇪",
            base_cost: 12_000.0,
            base_cps: 10.0,
            description: "Mines out cookie dough and chocolate chips.",
        },
        BuildingDef {
            id: "factory",
            name: "Factory",
            icon: "󱏛",
            base_cost: 130_000.0,
            base_cps: 40.0,
            description: "Manufactures cookies by the thousands.",
        },
        BuildingDef {
            id: "bank",
            name: "Bank",
            icon: "󰷕",
            base_cost: 1_400_000.0,
            base_cps: 100.0,
            description: "Generates cookies from compound interest.",
        },
        BuildingDef {
            id: "temple",
            name: "Temple",
            icon: "󰠮",
            base_cost: 20_000_000.0,
            base_cps: 400.0,
            description: "Worships the ancient cookie gods.",
        },
        BuildingDef {
            id: "wizard_tower",
            name: "Wizard Tower",
            icon: "󱏕",
            base_cost: 330_000_000.0,
            base_cps: 6_666.0,
            description: "Casts cookies into existence.",
        },
    ]
}

// ── Upgrades ──────────────────────────────────────────────────────────────────

pub enum UnlockCondition {
    Always,
    OwnBuilding { id: &'static str, min: u32 },
    TotalCookies(f64),
    ClickUpgradeCount(u32),
}

pub enum Effect {
    ClickBonus(f64),
    ClickMultiplier(f64),
    BuildingMultiplier { id: &'static str, factor: f64 },
    AllCpsMultiplier(f64),
}

pub struct UpgradeDef {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub cost: f64,
    pub description: &'static str,
    pub unlock: UnlockCondition,
    pub effect: Effect,
}

pub fn upgrades() -> Vec<UpgradeDef> {
    vec![
        // ── Click upgrades (chain unlocks) ────────────────────────────────
        UpgradeDef {
            id: "reinforced_clicking",
            name: "Reinforced clicking",
            icon: "󰳽",
            cost: 100.0,
            description: "+1 cookie per click",
            unlock: UnlockCondition::Always,
            effect: Effect::ClickBonus(1.0),
        },
        UpgradeDef {
            id: "mouse_wheel",
            name: "Mouse wheel",
            icon: "󰳽",
            cost: 500.0,
            description: "+1 cookie per click",
            unlock: UnlockCondition::ClickUpgradeCount(1),
            effect: Effect::ClickBonus(1.0),
        },
        UpgradeDef {
            id: "plastic_mouse",
            name: "Plastic mouse",
            icon: "󰳽",
            cost: 10_000.0,
            description: "+2 cookies per click",
            unlock: UnlockCondition::ClickUpgradeCount(2),
            effect: Effect::ClickBonus(2.0),
        },
        UpgradeDef {
            id: "iron_mouse",
            name: "Iron mouse",
            icon: "󰳽",
            cost: 100_000.0,
            description: "Clicks are twice as powerful",
            unlock: UnlockCondition::ClickUpgradeCount(3),
            effect: Effect::ClickMultiplier(2.0),
        },
        UpgradeDef {
            id: "titanium_mouse",
            name: "Titanium mouse",
            icon: "󰳽",
            cost: 1_000_000.0,
            description: "Clicks are twice as powerful",
            unlock: UnlockCondition::ClickUpgradeCount(4),
            effect: Effect::ClickMultiplier(2.0),
        },
        // ── Cursor upgrades ───────────────────────────────────────────────
        UpgradeDef {
            id: "cursor_1",
            name: "Thousand fingers",
            icon: "󰳽",
            cost: 1_000.0,
            description: "Cursors are twice as effective",
            unlock: UnlockCondition::OwnBuilding {
                id: "cursor",
                min: 1,
            },
            effect: Effect::BuildingMultiplier {
                id: "cursor",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "cursor_2",
            name: "Million fingers",
            icon: "󰳽",
            cost: 10_000.0,
            description: "Cursors are twice as effective",
            unlock: UnlockCondition::OwnBuilding {
                id: "cursor",
                min: 10,
            },
            effect: Effect::BuildingMultiplier {
                id: "cursor",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "cursor_3",
            name: "Billion fingers",
            icon: "󰳽",
            cost: 100_000.0,
            description: "Cursors are twice as effective",
            unlock: UnlockCondition::OwnBuilding {
                id: "cursor",
                min: 25,
            },
            effect: Effect::BuildingMultiplier {
                id: "cursor",
                factor: 2.0,
            },
        },
        // ── Grandma upgrades ──────────────────────────────────────────────
        UpgradeDef {
            id: "grandma_1",
            name: "Grandma's recipe",
            icon: "󰜥",
            cost: 1_000.0,
            description: "Grandmas are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "grandma",
                min: 1,
            },
            effect: Effect::BuildingMultiplier {
                id: "grandma",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "grandma_2",
            name: "Steel-plated rolling pins",
            icon: "󰜥",
            cost: 5_000.0,
            description: "Grandmas are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "grandma",
                min: 5,
            },
            effect: Effect::BuildingMultiplier {
                id: "grandma",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "grandma_3",
            name: "Lubricated dentures",
            icon: "󰜥",
            cost: 50_000.0,
            description: "Grandmas are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "grandma",
                min: 25,
            },
            effect: Effect::BuildingMultiplier {
                id: "grandma",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "grandma_4",
            name: "Prune juice",
            icon: "󰜥",
            cost: 500_000.0,
            description: "Grandmas are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "grandma",
                min: 50,
            },
            effect: Effect::BuildingMultiplier {
                id: "grandma",
                factor: 2.0,
            },
        },
        // ── Farm upgrades ─────────────────────────────────────────────────
        UpgradeDef {
            id: "farm_1",
            name: "Cheap hoes",
            icon: "󱢲",
            cost: 11_000.0,
            description: "Farms are twice as productive",
            unlock: UnlockCondition::OwnBuilding { id: "farm", min: 1 },
            effect: Effect::BuildingMultiplier {
                id: "farm",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "farm_2",
            name: "Fertilizer",
            icon: "󱢲",
            cost: 55_000.0,
            description: "Farms are twice as productive",
            unlock: UnlockCondition::OwnBuilding { id: "farm", min: 5 },
            effect: Effect::BuildingMultiplier {
                id: "farm",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "farm_3",
            name: "Cookie trees",
            icon: "󱢲",
            cost: 550_000.0,
            description: "Farms are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "farm",
                min: 25,
            },
            effect: Effect::BuildingMultiplier {
                id: "farm",
                factor: 2.0,
            },
        },
        // ── Mine upgrades ─────────────────────────────────────────────────
        UpgradeDef {
            id: "mine_1",
            name: "Sugar gas",
            icon: "󱇪",
            cost: 130_000.0,
            description: "Mines are twice as productive",
            unlock: UnlockCondition::OwnBuilding { id: "mine", min: 1 },
            effect: Effect::BuildingMultiplier {
                id: "mine",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "mine_2",
            name: "Megadrill",
            icon: "󱇪",
            cost: 650_000.0,
            description: "Mines are twice as productive",
            unlock: UnlockCondition::OwnBuilding { id: "mine", min: 5 },
            effect: Effect::BuildingMultiplier {
                id: "mine",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "mine_3",
            name: "Ultradrill",
            icon: "󱇪",
            cost: 6_500_000.0,
            description: "Mines are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "mine",
                min: 25,
            },
            effect: Effect::BuildingMultiplier {
                id: "mine",
                factor: 2.0,
            },
        },
        // ── Factory upgrades ──────────────────────────────────────────────
        UpgradeDef {
            id: "factory_1",
            name: "Sturdier conveyor belts",
            icon: "󱏛",
            cost: 1_300_000.0,
            description: "Factories are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "factory",
                min: 1,
            },
            effect: Effect::BuildingMultiplier {
                id: "factory",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "factory_2",
            name: "Child labour",
            icon: "󱏛",
            cost: 6_500_000.0,
            description: "Factories are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "factory",
                min: 5,
            },
            effect: Effect::BuildingMultiplier {
                id: "factory",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "factory_3",
            name: "Sweatshop",
            icon: "󱏛",
            cost: 65_000_000.0,
            description: "Factories are twice as productive",
            unlock: UnlockCondition::OwnBuilding {
                id: "factory",
                min: 25,
            },
            effect: Effect::BuildingMultiplier {
                id: "factory",
                factor: 2.0,
            },
        },
        // ── Bank upgrades ─────────────────────────────────────────────────
        UpgradeDef {
            id: "bank_1",
            name: "Taller tellers",
            icon: "󰷕",
            cost: 14_000_000.0,
            description: "Banks are twice as productive",
            unlock: UnlockCondition::OwnBuilding { id: "bank", min: 1 },
            effect: Effect::BuildingMultiplier {
                id: "bank",
                factor: 2.0,
            },
        },
        UpgradeDef {
            id: "bank_2",
            name: "Scissor-resistant credit cards",
            icon: "󰷕",
            cost: 70_000_000.0,
            description: "Banks are twice as productive",
            unlock: UnlockCondition::OwnBuilding { id: "bank", min: 5 },
            effect: Effect::BuildingMultiplier {
                id: "bank",
                factor: 2.0,
            },
        },
        // ── Global upgrades ───────────────────────────────────────────────
        UpgradeDef {
            id: "global_1",
            name: "Broken timepiece",
            icon: "󰞌",
            cost: 1_000_000.0,
            description: "All production +10%",
            unlock: UnlockCondition::TotalCookies(1_000_000.0),
            effect: Effect::AllCpsMultiplier(1.1),
        },
        UpgradeDef {
            id: "global_2",
            name: "Wormhole",
            icon: "󰞌",
            cost: 100_000_000.0,
            description: "All production +10%",
            unlock: UnlockCondition::TotalCookies(100_000_000.0),
            effect: Effect::AllCpsMultiplier(1.1),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_ids_are_unique() {
        let ids: Vec<_> = buildings().iter().map(|b| b.id).collect();
        let mut seen = std::collections::HashSet::new();
        for id in &ids {
            assert!(seen.insert(*id), "duplicate building id: {id}");
        }
    }

    #[test]
    fn upgrade_ids_are_unique() {
        let ids: Vec<_> = upgrades().iter().map(|u| u.id).collect();
        let mut seen = std::collections::HashSet::new();
        for id in &ids {
            assert!(seen.insert(*id), "duplicate upgrade id: {id}");
        }
    }

    #[test]
    fn next_cost_first_purchase_equals_base() {
        let cursor = buildings().into_iter().find(|b| b.id == "cursor").unwrap();
        assert_eq!(cursor.next_cost(0), cursor.base_cost);
    }

    #[test]
    fn next_cost_scales_by_1_15() {
        let cursor = buildings().into_iter().find(|b| b.id == "cursor").unwrap();
        let expected = cursor.base_cost * 1.15;
        assert!((cursor.next_cost(1) - expected).abs() < 0.001);
    }

    #[test]
    fn all_eight_buildings_present() {
        let ids: std::collections::HashSet<_> = buildings().iter().map(|b| b.id).collect();
        for expected in [
            "cursor",
            "grandma",
            "farm",
            "mine",
            "factory",
            "bank",
            "temple",
            "wizard_tower",
        ] {
            assert!(ids.contains(expected), "missing building: {expected}");
        }
    }
}
