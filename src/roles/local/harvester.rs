use log::warn;
use screeps::{Source, HasPosition, Creep, SharedCreepProperties};

use crate::{memory::CreepMemory, movement};

pub fn harvest(creep: &Creep, creepmem: &mut CreepMemory, source: Source) {
    let name = creep.name();
    if creep.pos().is_near_to(source.pos()) {
        creep.harvest(&source).unwrap_or_else(|e| {
            warn!("couldn't harvest: {:?}", e);
        });
    } else {
        movement::creep::move_to(&name, creepmem, source.pos())
    }
}