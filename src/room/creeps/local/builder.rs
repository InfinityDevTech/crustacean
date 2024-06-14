use screeps::{ConstructionSite, Creep, HasPosition, Part, ResourceType, SharedCreepProperties, StructureObject};

use crate::{
    memory::{CreepMemory, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::{hauling::HaulingType, CachedRoom, RoomCache}, traits::creep::CreepExtensions
};

use super::hauler::execute_order;

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();
    let needs_energy = creep_memory.needs_energy.unwrap_or(false);

    if creep.spawning() {
        return;
    }

    if needs_energy || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let _ = creep.say("📋", false);
        find_energy(creep, memory, cached_room);
    } else {
        build(creep, creep_memory, cached_room)
    }
}

pub fn build(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut CachedRoom) {
    for repairable in cache.structures.needs_repair.clone() {
        if repairable.as_repairable().unwrap().hits() as f32 > (repairable.as_repairable().unwrap().hits_max() as f32 * 0.10) {
            continue;
        }

        if repairable.pos().get_range_to(creep.pos()) > 1 {
            let _ = creep.say("🚚", false);
            creep.better_move_to(creepmem, cache, repairable.pos(), 1, MoveOptions::default());
            return;
        } else {
            let _ = creep.say("🔨", false);
            let _ = creep.repair(repairable.as_repairable().unwrap());
            cache.stats.energy.spending_repair += energy_spent_repairing(creep, &repairable);
            return;
        }
    }


    let sites = cache.structures.construction_sites.clone();

    let mut site_clone = sites.clone();
    site_clone.retain(|s| s.structure_type() != screeps::StructureType::Road);

    let sites = if site_clone.is_empty() {
        sites
    } else {
        site_clone
    };

    if !sites.is_empty() {
        if let Some(site) = sites.first() {

            if site.pos().get_range_to(creep.pos()) > 1 {
                let _ = creep.say("🚚", false);
                creep.better_move_to(creepmem, cache, site.pos(), 1, MoveOptions::default());
            } else {
                let _ = creep.say("🔨", false);
                let _ = creep.build(site);
                cache.stats.energy.spending_construction += energy_spent_building(creep, site);
            }

        }
    }

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.needs_energy = Some(true);
    }
}

pub fn find_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut CachedRoom) {
    let creepmem = memory.creeps.get(&creep.name()).unwrap();

    let task = &creepmem.hauling_task.clone();

    if let Some(task) = task {
        let creepmem_mut = memory.creeps.get_mut(&creep.name()).unwrap();
        execute_order(creep, creepmem_mut, cache, task);
    } else {
        let new_order = cache.hauling.find_new_order(
            creep,
            memory,
            Some(ResourceType::Energy),
            vec![HaulingType::Offer, HaulingType::Pickup],
            &mut cache.heap_cache
        );

        if let Some(order) = new_order {
            execute_order(
                creep,
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cache,
                &order,
            );
        }
    }
}

pub fn energy_spent_building(creep: &Creep, csite: &ConstructionSite) -> u32 {
    let work_parts = creep.body().iter().filter(|p| p.part() == Part::Work && p.hits() > 0).count() as u32;
    let work = creep.store().get_used_capacity(Some(ResourceType::Energy)).min(work_parts * 5);

    
    work.min(csite.progress_total() - csite.progress())
}

pub fn energy_spent_repairing(creep: &Creep, repairable: &StructureObject) -> u32 {
    let work_parts = creep.body().iter().filter(|p| p.part() == Part::Work && p.hits() > 0).count() as u32;
    let work = creep.store().get_used_capacity(Some(ResourceType::Energy)).min(work_parts * 5);

    let repairable = repairable.as_repairable().unwrap();

    work.min(repairable.hits_max() - repairable.hits())
}
