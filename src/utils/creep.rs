use screeps::{Creep, Part};

use crate::room::creeps::local::{builder, source_miner, upgrader};

pub fn creep_tired(creep: &Creep) -> bool {
    creep.fatigue() > 0
}

pub fn creep_parts_of_type(creep: &Creep, part: Part) -> u32 {
    creep.body().iter().filter(|b| b.part() == part).count() as u32
}