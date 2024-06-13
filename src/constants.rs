use std::sync::OnceLock;

use enum_map::{enum_map, Enum, EnumMap};
use screeps::Part;

#[derive(Debug, Enum)]
pub enum PartsCost {
    Move,
    Work,
    Carry,
    Attack,
    RangedAttack,
    Heal,
    Claim,
    Tough,
}

pub fn part_costs() -> &'static EnumMap<PartsCost, u32> {
    static PART_COSTS: OnceLock<EnumMap<PartsCost, u32>> = OnceLock::new();
    PART_COSTS.get_or_init(|| enum_map! {
        PartsCost::Move => 50,
        PartsCost::Work => 100,
        PartsCost::Carry => 50,
        PartsCost::Attack => 80,
        PartsCost::RangedAttack => 150,
        PartsCost::Heal => 250,
        PartsCost::Claim => 600,
        PartsCost::Tough => 10,
    })
}
