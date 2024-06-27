use screeps::{ConstructionSite, Creep, HasPosition, Part, ResourceType, SharedCreepProperties, StructureObject};

use crate::{
    memory::ScreepsMemory, movement::move_target::MoveOptions, room::cache::tick_cache::{hauling::{HaulTaskRequest, HaulingType}, RoomCache}, traits::creep::CreepExtensions
};

use super::{hauler::execute_order, upgrader::run_upgrader};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_builder(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    let needs_energy = creep_memory.needs_energy.unwrap_or(false);

    if creep.spawning() {
        return;
    }

    if needs_energy || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        find_energy(creep, memory, cache);
    } else {
        build(creep, memory, cache)
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn build(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creepmem = memory.creeps.get_mut(&creep.name()).unwrap();
    let room_cache = cache.rooms.get_mut(&creepmem.owning_room).unwrap();
    let sites = room_cache.structures.construction_sites.clone();

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
                creep.better_move_to(creepmem, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), site.pos(), 1, MoveOptions::default());
            } else {
                let _ = creep.say("🔨", false);
                let _ = creep.build(site);
                room_cache.stats.energy.spending_construction += energy_spent_building(creep, site);
            }

        }
    } else {
        run_upgrader(creep, memory, cache);
        return;
    }

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.needs_energy = Some(true);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creepmem = memory.creeps.get_mut(&creep.name()).unwrap();

    let task = &creepmem.hauling_task.clone();

    if let Some(task) = task {
        let _ = creep.say("📋", false);

        execute_order(creep, creepmem, cache, task);
    } else {
        let _ = creep.say("🔋", false);

        let room_cache = cache.rooms.get_mut(&creepmem.owning_room).unwrap();

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
