#![allow(dead_code)]
use screeps::{find, Creep, HasPosition, ResourceType, SharedCreepProperties};

use crate::{memory::{CreepMemory, ScreepsMemory}, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory) {
    let creep_memory = memory.get_creep_mut(creep.name().as_str());
    let needs_energy = creep_memory.n_e.unwrap_or(false);

    if needs_energy || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        find_energy(creep, creep_memory)
    } else {
        build(creep, creep_memory)
    }
}

pub fn build(creep: &Creep, creepmem: &mut CreepMemory) {
    let closest_site = creep.pos().find_closest_by_range(find::CONSTRUCTION_SITES);
        if let Some(site) = closest_site {
            if creep.pos().is_near_to(site.clone().pos()) {
                let _ = creep.build(&site);
            } else {
                creep.better_move_to(creepmem, site.pos(), 1)
            }
        }
}

pub fn find_energy(creep: &Creep, creepmem: &mut CreepMemory) {
    let closest_energy = creep
            .pos()
            .find_closest_by_range(find::DROPPED_RESOURCES);
        if let Some(energy) = closest_energy {
            if creep.pos().is_near_to(energy.clone().pos()) {
                let _ = creep.pickup(&energy);
            } else {
                creep.better_move_to(creepmem, energy.pos(), 1)
            }
        }
}
