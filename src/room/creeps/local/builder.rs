use screeps::{
    ConstructionSite, Creep, HasPosition, Part, ResourceType, SharedCreepProperties,
};

use crate::{
    memory::ScreepsMemory,
    movement::move_target::MoveOptions,
    room::cache::tick_cache::{
        hauling::{HaulTaskRequest, HaulingType},
        RoomCache,
    },
    traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking},
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
        creep_memory.needs_energy = None;
        creep_memory.hauling_task = None;

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

    if let Some(storage) = &room_cache.structures.storage {
        if storage.store().get_used_capacity(Some(ResourceType::Energy)) < 10000 {
            site_clone.retain(|s| s.structure_type() != screeps::StructureType::Rampart);
        }
    }

    let sites = if site_clone.is_empty() {
        sites
    } else {
        site_clone
    };

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.needs_energy = Some(true);
    }

    if !sites.is_empty() {
        if let Some(site) = sites.first() {
            if site.pos().get_range_to(creep.pos()) > 3 {
                creep.bsay("🚚", false);
                creep.better_move_to(
                    memory,
                    cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
                    site.pos(),
                    3,
                    MoveOptions::default(),
                );
            } else {
                creep.bsay("🔨", false);
                let _ = creep.ITbuild(site);
                room_cache.stats.energy.spending_construction += energy_spent_building(creep, site);
            }
        }
    } else {
        run_upgrader(creep, memory, cache);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creepmem = memory.creeps.get_mut(&creep.name()).unwrap();

    let task = &creepmem.hauling_task.clone();

    let room_cache = cache.rooms.get_mut(&creepmem.owning_room).unwrap();

    if let Some(storage) = &room_cache.structures.storage {
        if storage.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
            if !creep.pos().is_near_to(storage.pos()) {
                creep.bsay("🚚", false);
                creep.better_move_to(
                    memory,
                    room_cache,
                    storage.pos(),
                    1,
                    MoveOptions::default(),
                );
            } else {
                creep.bsay("🔋", false);
                let _ = creep.ITwithdraw(storage, ResourceType::Energy, None);
            }

            return;
        }
    }

    if room_cache.structures.containers.fast_filler.is_some() {
        let mut run = true;
        if let Some((_spawn, spawn_id)) = &room_cache.structures.spawns.clone().into_iter().next() {
            if spawn_id
                .store()
                .get_free_capacity(Some(ResourceType::Energy))
                > 0
            {
                run = false;
            }
        }

        if run {
            let mut containers = room_cache
                .structures
                .containers
                .fast_filler
                .as_ref()
                .unwrap()
                .clone();
            containers.sort_by_key(|c| c.store().get_free_capacity(Some(ResourceType::Energy)));

            if let Some(container) = containers.first() {
                if !creep.pos().is_near_to(container.pos()) {
                    creep.bsay("🚚", false);
                    creep.better_move_to(
                        memory,
                        room_cache,
                        container.pos(),
                        1,
                        MoveOptions::default(),
                    );
                } else {
                    creep.bsay("🔋", false);
                    let _ = creep.ITwithdraw(container, ResourceType::Energy, None);
                }

                return;
            }
        }
    }

    if let Some(task) = task {
        creep.bsay("📋", false);

        execute_order(creep, memory, cache, task);
    } else {
        creep.bsay("🔋", false);

        room_cache.hauling.wanting_orders.push(
            HaulTaskRequest::default()
                .creep_name(creep.name())
                .resource_type(ResourceType::Energy)
                .haul_type(vec![
                    HaulingType::Pickup,
                    HaulingType::Withdraw,
                    HaulingType::Offer,
                ])
                .finish(),
        );
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn energy_spent_building(creep: &Creep, csite: &ConstructionSite) -> u32 {
    let work_parts = creep
        .body()
        .iter()
        .filter(|p| p.part() == Part::Work && p.hits() > 0)
        .count() as u32;
    let work = creep
        .store()
        .get_used_capacity(Some(ResourceType::Energy))
        .min(work_parts * 5);

    work.min(csite.progress_total() - csite.progress())
}
