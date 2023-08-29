use log::warn;
use screeps::{Source, HasPosition, Creep, ResourceType};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions};

pub fn harvest(creep: &Creep, creepmem: &mut CreepMemory, source: Source) {
    if creep.pos().is_near_to(source.pos()) {

        if creep.store().get_free_capacity(Some(ResourceType::Energy)) > creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32 {

            let _ = creep.drop(ResourceType::Energy, Some(creep.store().get_used_capacity(Some(ResourceType::Energy))));

        } else {

        creep.harvest(&source).unwrap_or_else(|e| {
            warn!("couldn't harvest: {:?}", e);
        });

    }
    } else {

        creep.better_move_to(creepmem, source.pos(), 1)

    }
}