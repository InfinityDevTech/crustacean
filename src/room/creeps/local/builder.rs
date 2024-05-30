use screeps::{Creep, HasPosition, ResourceType, SharedCreepProperties};

use crate::{memory::{CreepMemory, ScreepsMemory}, room::cache::{hauling::HaulingType, RoomCache}, traits::creep::CreepExtensions};

use super::hauler::execute_order;

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let needs_energy = creep_memory.needs_energy.unwrap_or(false);

    if creep.spawning() {
        return;
    }

    if needs_energy || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let _ = creep.say("📋", false);
        find_energy(creep, memory, cache);
    } else {
        build(creep, creep_memory, cache)
    }
}

pub fn build(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut RoomCache) {
    let sites = &cache.structures.construction_sites;

    if !sites.is_empty() {
        let site = sites.first().unwrap();
        if site.pos().get_range_to(creep.pos()) > 1 {
            let _ = creep.say("🚚", false);
            creep.better_move_to(creepmem, cache, site.pos(), 1);
        } else {
            let _ = creep.say("🔨", false);
            let _ = creep.build(site);
        }
    }

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.needs_energy = Some(true);
    }
}

pub fn find_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creepmem = memory.creeps.get(&creep.name()).unwrap();

    let task = &creepmem.hauling_task.clone();

    if let Some(task) = task {

        let creepmem_mut = memory.creeps.get_mut(&creep.name()).unwrap();
        execute_order(creep, creepmem_mut, cache, task);

    } else {
        let new_order = cache.hauling.find_new_order(creep, memory, Some(ResourceType::Energy), vec![HaulingType::Offer, HaulingType::Pickup]);

        if let Some(order) = new_order {
            execute_order(creep, memory.creeps.get_mut(&creep.name()).unwrap(), cache, &order);
        }

    }
}
