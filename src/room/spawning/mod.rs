use std::cmp::{self, min};

use log::info;
use screeps::{find, game, Part, Room, SharedCreepProperties};
use spawn_manager::SpawnManager;

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    utils::get_body_cost,
};

use super::cache::{self, tick_cache::{CachedRoom, RoomCache}};

pub mod creep_sizing;
pub mod spawn_manager;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn handle_spawning(room: &Room, cache: &mut RoomCache, memory: &mut ScreepsMemory) {
    let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

    let mut spawn_manager = SpawnManager::new(&room.name(), room_cache);

    miner(room, room_cache, &mut spawn_manager);
    hauler(room, room_cache, &mut spawn_manager, memory);
    fast_filler(room, room_cache, &mut spawn_manager);
    flag_attacker(room, room_cache, &mut spawn_manager);
    builder(room, room_cache, &mut spawn_manager);
    upgrader(room, room_cache, &mut spawn_manager);
    scout(room, room_cache, &mut spawn_manager);

    remote_harvester(room, cache, memory, &mut spawn_manager);

    spawn_manager.run_spawning(room, memory);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn flag_attacker(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    if let Some(flag) = game::flags().get("bulldozeRoom".to_string()) {
        let attackers = cache
            .creeps
            .creeps_of_role
            .get(&Role::Bulldozer)
            .unwrap_or(&Vec::new())
            .len();

        let unclaimer = cache
            .creeps
            .creeps_of_role
            .get(&Role::Unclaimer)
            .unwrap_or(&Vec::new())
            .len();

        let mut should_spawn_unclaimer = false;
        if let Some(room) = flag.room() {
            if room.find(find::HOSTILE_CREEPS, None).is_empty() && room.find(find::HOSTILE_SPAWNS, None).is_empty() && room.find(find::HOSTILE_STRUCTURES, None).is_empty() {
                should_spawn_unclaimer = true;
            }
        }

        if attackers >= 4 && unclaimer >= 1 {
            return;
        }

        if attackers < 4 && !should_spawn_unclaimer {
            let mut body = vec![Part::Move, Part::Move, Part::Heal];
            let max_energy = room.energy_capacity_available();
            let mut cost = 350;

            let mut isnt_potato = false;

            while cost < max_energy {
                if cost + 130 > max_energy {
                    break;
                }

                isnt_potato = true;

                body.push(Part::Attack);
                body.push(Part::Move);
                cost += 130;
            }

            if isnt_potato {
                spawn_manager.create_spawn_request(Role::Bulldozer, body, 4.0, cost, None, None);
            }
        } else if unclaimer < 1 && !unclaimer > 3 && should_spawn_unclaimer {
            let mut body = vec![Part::Move, Part::Move];
            let max_energy = room.energy_capacity_available();
            let mut cost = 100;

            let mut isnt_potato = false;

            while cost < max_energy {
                if cost + 650 > max_energy {
                    break;
                }

                isnt_potato = true;

                body.push(Part::Claim);
                body.push(Part::Move);
                cost += 650;
            }

            if isnt_potato {
                spawn_manager.create_spawn_request(Role::Unclaimer, body, 4.0, cost, None, None);
            }
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn scout(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let body = vec![Part::Move];
    let cost = get_body_cost(&body);

    let scouts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Scout)
        .unwrap_or(&Vec::new())
        .len();

    let has_observer = cache.structures.observer.is_some();

    let count = if has_observer {
        1
    } else {
        2
    };

    if scouts >= count {
        return;
    }

    spawn_manager.create_spawn_request(Role::Scout, body, 4.0, cost, None, None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn builder(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let building_work_parts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Builder)
        .unwrap_or(&Vec::new())
        .iter().map(|c| game::creeps().get(c.to_string()).unwrap().body().iter().filter(|p| p.part() == Part::Work).count() as u32)
        .sum::<u32>();

    let construction_sites = cache.structures.construction_sites.len();

    if construction_sites == 0 {
        return;
    }

    let desired_work_parts = cmp::max(construction_sites / 3, 12);
    if building_work_parts as usize >= desired_work_parts {
        return;
    }

    let body = crate::room::spawning::creep_sizing::builder(room, cache);
    let cost = get_body_cost(&body);

    spawn_manager.create_spawn_request(Role::Builder, body, 4.0, cost, None, None);

}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn upgrader(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let upgraders = cache
        .creeps
        .creeps_of_role
        .get(&Role::Upgrader)
        .unwrap_or(&Vec::new())
        .len();

    let body = crate::room::spawning::creep_sizing::upgrader(room, cache);
    let cost = get_body_cost(&body);

    if body.is_empty() {
        return;
    }

    spawn_manager.create_spawn_request(Role::Upgrader, body, 4.0, cost, None, None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn hauler(
    room: &Room,
    cache: &mut CachedRoom,
    spawn_manager: &mut SpawnManager,
    memory: &mut ScreepsMemory,
) {
    let current_hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    let priority = if current_hauler_count < 2 {
        10.0
    } else {
        4.5
    };

    let mut dropped_count = 0;
    for resource in &cache.resources.dropped_energy {
        dropped_count += resource.amount();
    }

    let energy_in_room = cache.resources.energy_in_storing_structures + dropped_count;

    let haulers_to_make = if energy_in_room > 500000 {
        energy_in_room / 200000
    } else if energy_in_room > 100000 {
        energy_in_room / 25000
    } else if energy_in_room > 50000 {
        energy_in_room / 10000
    } else if energy_in_room > 10000 {
        energy_in_room / 5000
    } else {
        let count = energy_in_room / 500;

        if count == 0 {
            6
        } else {
            count
        }
    };

    if current_hauler_count as u32 >= haulers_to_make {
        return;
    }

    let body = crate::room::spawning::creep_sizing::hauler(room, cache);
    let cost = get_body_cost(&body);

    let creepmem = CreepMemory {
        owning_room: room.name(),
        needs_energy: Some(true),
        ..Default::default()
    };

    spawn_manager.create_spawn_request(Role::Hauler, body, priority, cost, Some(creepmem), None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn fast_filler(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let fast_filler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::FastFiller)
        .unwrap_or(&Vec::new())
        .len();

    if fast_filler_count >= 2 {
        return;
    }

    let body = vec![Part::Carry, Part::Move];
    let cost = get_body_cost(&body);

    spawn_manager.create_spawn_request(Role::FastFiller, body, 50.0, cost, None, None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn miner(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let miner_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Miner)
        .unwrap_or(&Vec::new())
        .len();
    let hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    for source in &cache.resources.sources {
        let parts_needed = source.parts_needed();

        if parts_needed == 0 || source.creeps.len() >= source.calculate_mining_spots(room).into() {
            continue;
        }
        let parts = crate::room::spawning::creep_sizing::miner(room, cache, parts_needed);
        let cost = get_body_cost(&parts);

        let mut priority = 2.0;

        if miner_count < hauler_count {
            priority -= 1.0;
        }
        priority += (parts_needed as f64) * 0.75;

        if miner_count < cache.resources.sources.len() {
            priority += 50.0;
        }

        let index = &cache
            .resources
            .sources
            .iter()
            .position(|s| s.id == source.id)
            .unwrap();

        let creep_memory = CreepMemory {
            owning_room: room.name(),
            task_id: Some((*index).try_into().unwrap()),
            ..Default::default()
        };

        spawn_manager.create_spawn_request(
            Role::Miner,
            parts,
            priority,
            cost,
            Some(creep_memory),
            None,
        );
    }
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn remote_harvester(room: &Room, cache: &mut RoomCache, memory: &mut ScreepsMemory, spawn_manager: &mut SpawnManager) {
    let remotes = cache.rooms.get_mut(&room.name()).unwrap().remotes.clone();
    for remote in remotes {
        if let Some(remote_room) = game::rooms().get(remote) {
            cache.create_if_not_exists(&remote_room, memory, None);
            let cache = cache.rooms.get_mut(&remote_room.name()).unwrap();

            info!("Room {:?} has {} sources", remote_room, cache.resources.sources.len());

            for source in cache.resources.sources.iter() {
                let parts_needed = source.parts_needed();

                if parts_needed == 0 || source.creeps.len() >= source.calculate_mining_spots(&remote_room).into() {
                    continue;
                }

                info!("Parts needed {} for room {} and source {}", parts_needed, remote_room.name(), source.id);

                let parts = crate::room::spawning::creep_sizing::miner(&remote_room, cache, parts_needed);
                let cost = get_body_cost(&parts);

                let mut priority = 0.0;

                priority += (parts_needed as f64) * 1.5;

                let index = &cache
                    .resources
                    .sources
                    .iter()
                    .position(|s| s.id == source.id)
                    .unwrap();

                let creep_memory = CreepMemory {
                    owning_room: room.name(),
                    owning_remote: Some(remote),
                    task_id: Some((*index).try_into().unwrap()),
                    ..Default::default()
                };

                spawn_manager.create_spawn_request(
                    Role::RemoteHarvester,
                    parts,
                    priority,
                    cost,
                    Some(creep_memory),
                    None,
                );
            }
        } else {
            let cached_room = cache.rooms.get_mut(&room.name()).unwrap();
            let existing_physical_observers = cached_room
                .creeps
                .creeps_of_role
                .get(&Role::PhysicalObserver);

            if let Some(existing_observers) = existing_physical_observers {
                for existing_observer in existing_observers.iter() {
                    let creep = game::creeps().get(existing_observer.to_string()).unwrap();
                    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

                    if creep_memory.scout_target.is_none() || creep.room().unwrap().name() == creep_memory.scout_target.unwrap() {
                        creep_memory.scout_target = Some(remote);

                        return;
                    }
                }
            } else {

                let body = vec![Part::Move];
                let cost = get_body_cost(&body);

                let creep_memory = CreepMemory {
                    owning_room: room.name(),
                    scout_target: Some(remote),
                    ..Default::default()
                };

                spawn_manager.create_spawn_request(Role::PhysicalObserver, body, 10.0, cost, Some(creep_memory), None);
            }
        }
    }
}