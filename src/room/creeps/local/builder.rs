use screeps::{
    ConstructionSite, Creep, HasPosition, Part, Position, ResourceType, RoomCoordinate, RoomName, SharedCreepProperties
};

use crate::{
    memory::ScreepsMemory,
    movement::move_target::MoveOptions,
    room::cache::{
        hauling::{HaulTaskRequest, HaulingType},
        RoomCache,
    },
    traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking},
    utils::under_storage_gate,
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

pub fn get_all_remote_csites(
    main_room: &RoomName,
    room_cache: &RoomCache,
    memory: &ScreepsMemory,
) -> Vec<ConstructionSite> {
    let mut sites = Vec::new();

    if let Some(memory) = memory.rooms.get(main_room) {
        for remote in &memory.remotes {
            if let Some(cache) = room_cache.rooms.get(remote) {
                let remote_sites = cache.structures.construction_sites.clone();

                for csite in remote_sites {
                    if csite.my() {
                        sites.push(csite);
                    }
                }
            }
        }
    }

    sites
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn build(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creepmem = memory.creeps.get_mut(&creep.name()).unwrap();
    let room_cache = cache.rooms.get_mut(&creepmem.owning_room).unwrap();
    let mut sites = room_cache.structures.construction_sites.clone();
    //sites.sort_by_key(|s| s.pos().get_range_to(creep.pos()));

    let room = &creepmem.owning_room.clone();
    drop(creepmem);

    let sites = if room_cache.structures.construction_sites.is_empty() {;
        get_all_remote_csites(room, cache, memory)
    } else {
        room_cache.structures.construction_sites.clone()
    };

    let creepmem = memory.creeps.get_mut(&creep.name()).unwrap();

    /*if creep.room().unwrap().name() != creepmem.owning_room {
        let room = creepmem.owning_room;

        creep.better_move_to(
            memory,
            cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
            Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(25).unwrap(), room),
            23,
            MoveOptions::default(),
        );

        return;
    }*/

    let creepmem = memory.creeps.get_mut(&creep.name()).unwrap();
    let room_cache = cache.rooms.get_mut(&creepmem.owning_room).unwrap();

    let mut site_clone = sites.clone();
    site_clone.retain(|s| s.structure_type() != screeps::StructureType::Road);

    if let Some(storage) = &room_cache.structures.storage {
        if storage
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            < 10000
        {
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
            if site.pos().get_range_to(creep.pos()) > 3
                || creep.room().unwrap().name() != site.pos().room_name()
            {
                creep.bsay("🚚", false);
                if site.pos().room_name() != creep.room().unwrap().name() {
                    creep.better_move_to(
                        memory,
                        cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
                        site.pos(),
                        0,
                        MoveOptions::default(),
                    );
                } else {
                    creep.better_move_to(
                        memory,
                        cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
                        site.pos(),
                        3,
                        MoveOptions::default(),
                    );
                }
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

    /*for resource in &room_cache.resources.dropped_energy {
        if creep.pos().get_range_to(resource.pos()) <= 3 && resource.resource_type() == ResourceType::Energy {
            if creep.pos().is_near_to(resource.pos()) {
                let _ = creep.pickup(resource);
            } else {
                creep.better_move_to(memory, room_cache, resource.pos(), 1, MoveOptions::default());

                return;
            }
        }
    }*/

    if let Some(storage) = &room_cache.structures.storage {
        if !under_storage_gate(room_cache, 0.7) {
            if !creep.pos().is_near_to(storage.pos()) {
                creep.bsay("🚚", false);
                let pos = storage.pos();
                creep.better_move_to(memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), pos, 1, MoveOptions::default());
            } else {
                creep.bsay("🔋", false);
                let _ = creep.ITwithdraw(storage, ResourceType::Energy, None);
            }

            return;
        }
    }

    if room_cache.structures.containers().fast_filler.is_some() {
        let mut run = true;
        let mut highest = 0;

        for container in room_cache.structures.containers().fast_filler.as_ref().unwrap() {
            if container.store().get_used_capacity(Some(ResourceType::Energy)) > highest {
                highest = container.store().get_used_capacity(Some(ResourceType::Energy));
            }
        }

        if highest < 500 {
            run = false;
        }

        if run {
            let mut containers = room_cache
                .structures
                .containers()
                .fast_filler
                .as_ref()
                .unwrap()
                .clone();
            containers.sort_by_key(|c| c.store().get_used_capacity(Some(ResourceType::Energy)));

            if let Some(container) = containers.first() {
                if !creep.pos().is_near_to(container.pos()) {
                    creep.bsay("🚚", false);
                    creep.better_move_to(
                        memory,
                        cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
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

    if room_cache.structures.containers().fast_filler.is_none() && room_cache.rcl >= 2 {
        //room_cache.hauling.create_order(creep.try_raw_id().unwrap(), None, Some(ResourceType::Energy), Some(creep.store().get_capacity(Some(ResourceType::Energy))), 25.0, HaulingType::Transfer);

        //return;
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
                .maintain_room(creepmem.owning_room)
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
