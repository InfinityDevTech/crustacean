use log::warn;
use screeps::{Source, HasPosition, Creep, SharedCreepProperties, ResourceType};

use crate::{memory::CreepMemory, movement};

pub fn harvest(creep: &Creep, creepmem: &mut CreepMemory, source: Source) {
    let name = creep.name();
    if creep.pos().is_near_to(source.pos()) {
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) > creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32 {
            let _ = creep.drop(ResourceType::Energy, Some(creep.store().get_used_capacity(Some(ResourceType::Energy))));
        } else {
        creep.harvest(&source).unwrap_or_else(|e| {
            warn!("couldn't harvest: {:?}", e);
        });
    }
    } else {
        movement::creep::move_to(&name, creepmem, source.pos())
    }
}