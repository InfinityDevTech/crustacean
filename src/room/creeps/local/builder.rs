use screeps::{ConstructionSite, Creep, HasPosition, Part, Position, ResourceType, RoomCoordinate, SharedCreepProperties, StructureObject};

use crate::{
    memory::{CreepMemory, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::{self, tick_cache::{hauling::{HaulTaskRequest, HaulingType}, CachedRoom, RoomCache}}, traits::creep::CreepExtensions
};

use super::hauler::execute_order;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_builder(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep.room().unwrap().name());
    if cached_room.is_none() {
        return;
    }
    let cached_room = cached_room.unwrap();

    let needs_energy = creep_memory.needs_energy.unwrap_or(false);

    if creep.spawning() {
        return;
    }

    if needs_energy || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let _ = creep.say("📋", false);
        find_energy(creep, memory, cache);
    } else if creep.room().unwrap().name() != creep_memory.owning_room {
        let _ = creep.say("🚚", false);
        creep.better_move_to(creep_memory, cached_room, Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(25).unwrap(), creep_memory.owning_room), 23, MoveOptions::default());
    } else {
        build(creep, creep_memory, cached_room)
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creepmem = memory.creeps.get(&creep.name()).unwrap();
    let room_cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    let task = &creepmem.hauling_task.clone();

    if let Some(task) = task {
        let creepmem_mut = memory.creeps.get_mut(&creep.name()).unwrap();
        execute_order(creep, creepmem_mut, room_cache, task);
    } else {
        room_cache.hauling.wanting_orders.push(HaulTaskRequest::default().creep_name(creep.name()).resource_type(ResourceType::Energy).haul_type(vec![HaulingType::Pickup, HaulingType::Withdraw, HaulingType::Offer]).finish());
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn energy_spent_building(creep: &Creep, csite: &ConstructionSite) -> u32 {
    let work_parts = creep.body().iter().filter(|p| p.part() == Part::Work && p.hits() > 0).count() as u32;
    let work = creep.store().get_used_capacity(Some(ResourceType::Energy)).min(work_parts * 5);

    
    work.min(csite.progress_total() - csite.progress())
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn energy_spent_repairing(creep: &Creep, repairable: &StructureObject) -> u32 {
    let work_parts = creep.body().iter().filter(|p| p.part() == Part::Work && p.hits() > 0).count() as u32;
    let work = creep.store().get_used_capacity(Some(ResourceType::Energy)).min(work_parts * 5);

    let repairable = repairable.as_repairable().unwrap();

    work.min(repairable.hits_max() - repairable.hits())
}
