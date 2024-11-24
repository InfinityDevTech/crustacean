
use std::sync::OnceLock;

use enum_map::{enum_map, Enum, EnumMap};
use screeps::{Part, ResourceType};

// navigator.clipboard.writeText(`$TO_COPY`);
// Hmmm, I want to optimize this so it automatically works.
// Specifically, button to copy. 
pub const COPY_TEXT: &str = "
<script>function selectText() { const text = document.getElementById(`to-select-$TIME`); text.focus(); text.select(); }</script>
<div style='display: flex; flex-direction: row; justify-content: flex-start; margin: 0; padding: 0;'>
    <input value='$TO_COPY$' type='text' id='to-select-$TIME' style='color: red; border-radius: 7px; margin: 0; padding: 0;'/>
    <button style='color: black; border-radius: 7px; margin: 0; padding: 0;' onclick='selectText()'> Copy profiler output </button>
</div>
";

pub const WALKABLE_STRUCTURES: [screeps::StructureType; 3] = [
    screeps::StructureType::Road,
    screeps::StructureType::Container,
    screeps::StructureType::Rampart,
];

pub const INVADER_USERNAME: &str = "Invader";
pub const MMO_SHARD_NAMES: [&str; 4] = ["shard0", "shard1", "shard2", "shard3"];
pub const MAX_BUCKET: i32 = 10000;

pub const NO_RCL_PLACEABLES: [screeps::StructureType; 4] = [
    screeps::StructureType::Road,
    screeps::StructureType::Container,
    screeps::StructureType::Rampart,
    screeps::StructureType::InvaderCore,
];

pub const ROOM_AREA: usize = 2500;
pub const ROOM_SIZE: u8 = 50;

pub const ITERABLE_RESOURCES: [ResourceType; 57] = [
    ResourceType::Energy,
    ResourceType::Hydrogen,
    ResourceType::Oxygen,
    ResourceType::Utrium,
    ResourceType::Keanium,
    ResourceType::Lemergium,
    ResourceType::Zynthium,
    ResourceType::Catalyst,

    ResourceType::Hydroxide,
    ResourceType::ZynthiumKeanite,
    ResourceType::UtriumLemergite,
    ResourceType::Ghodium,

    ResourceType::UtriumHydride,
    ResourceType::UtriumOxide,
    ResourceType::KeaniumHydride,
    ResourceType::KeaniumOxide,
    ResourceType::LemergiumHydride,
    ResourceType::LemergiumOxide,
    ResourceType::ZynthiumHydride,
    ResourceType::ZynthiumOxide,
    ResourceType::GhodiumHydride,
    ResourceType::GhodiumOxide,

    ResourceType::UtriumAcid,
    ResourceType::UtriumAlkalide,
    ResourceType::KeaniumAcid,
    ResourceType::KeaniumAlkalide,
    ResourceType::LemergiumAcid,
    ResourceType::LemergiumAlkalide,
    ResourceType::ZynthiumAcid,
    ResourceType::ZynthiumAlkalide,
    ResourceType::GhodiumAcid,
    ResourceType::GhodiumAlkalide,

    ResourceType::CatalyzedUtriumAcid,
    ResourceType::CatalyzedUtriumAlkalide,
    ResourceType::CatalyzedKeaniumAcid,
    ResourceType::CatalyzedKeaniumAlkalide,
    ResourceType::CatalyzedLemergiumAcid,
    ResourceType::CatalyzedLemergiumAlkalide,
    ResourceType::CatalyzedZynthiumAcid,
    ResourceType::CatalyzedZynthiumAlkalide,
    ResourceType::CatalyzedGhodiumAcid,
    ResourceType::CatalyzedGhodiumAlkalide,

    ResourceType::Power,
    ResourceType::Ops,

    ResourceType::Metal,
    ResourceType::Silicon,
    ResourceType::Biomass,
    ResourceType::Mist,

    ResourceType::UtriumBar,
    ResourceType::LemergiumBar,
    ResourceType::ZynthiumBar,
    ResourceType::KeaniumBar,
    ResourceType::GhodiumMelt,
    ResourceType::Oxidant,
    ResourceType::Reductant,
    ResourceType::Purifier,
    ResourceType::Battery,
];

pub const PATHFINDER_MAX_ROOMS: u32 = 64;

pub const WORLD_SIZE: u8 = 255;

pub const WALL_MASK: u8 = 1;
pub const SWAMP_MASK: u8 = 2;

pub static HARVEST_POWER: u8 = 2;
pub static REPAIR_POWER: u8 = 100;
pub static DISMANTLE_POWER: u8 = 50;
pub static BUILD_POWER: u8 = 5;
pub static UPGRADE_POWER: u8 = 1;
pub static HOSTILE_PARTS: [Part; 4] = [Part::Attack, Part::RangedAttack, Part::Heal, Part::Work];

pub static CLAIM_LIFETIME: u32 = 600;
pub static CREEP_LIFETIME: u32 = 1500;

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