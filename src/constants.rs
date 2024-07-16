
use std::sync::OnceLock;

use enum_map::{enum_map, Enum, EnumMap};
use screeps::Part;

// navigator.clipboard.writeText(`$TO_COPY`);
pub const COPY_TEXT: &str = "
<script>function selectText() { const text = document.getElementById(`to-select-$TIME`); text.focus(); text.select(); }</script>
<div style='display: flex; flex-direction: row; justify-content: flex-start; margin: 0; padding: 0;'>
    <input value='$TO_COPY$' type='text' id='to-select-$TIME' style='color: red; border-radius: 7px; margin: 0; padding: 0;'/>
    <button style='color: black; border-radius: 7px; margin: 0; padding: 0;' onclick='selectText()'> Copy profiler output </button>
</div>
";

pub const PATHFINDER_MAX_ROOMS: u32 = 64;

pub const WORLD_SIZE: u8 = 255;

pub static HOSTILE_PARTS: [Part; 3] = [Part::Attack, Part::RangedAttack, Part::Heal];

pub fn part_attack_weight(part: &Part) -> u32 {
    match part {
        Part::Attack => 30,
        Part::RangedAttack => 10,
        Part::Heal => 12,
        _ => 0,
    }
}

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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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
