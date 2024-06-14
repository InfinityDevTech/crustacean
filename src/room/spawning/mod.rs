use std::cmp::min;

use log::info;
use screeps::{find, game, Part, Room};
use spawn_manager::SpawnManager;

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    utils::get_body_cost,
};

use super::cache::tick_cache::CachedRoom;

pub mod creep_sizing;
pub mod spawn_manager;

pub fn handle_spawning(room: &Room, room_cache: &mut CachedRoom, memory: &mut ScreepsMemory) {
    let mut spawn_manager = SpawnManager::new(&room.name(), room_cache);

    miner(room, room_cache, &mut spawn_manager);
    hauler(room, room_cache, &mut spawn_manager, memory);
    fast_filler(room, room_cache, &mut spawn_manager);
    flag_attacker(room, room_cache, &mut spawn_manager);
    builder(room, room_cache, &mut spawn_manager);
    upgrader(room, room_cache, &mut spawn_manager);
    scout(room, room_cache, &mut spawn_manager);

    spawn_manager.run_spawning(room, room_cache, memory);
}

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
            if room.find(find::HOSTILE_CREEPS, None).is_empty() && room.find(find::HOSTILE_SPAWNS, None).is_empty() {
                should_spawn_unclaimer = true;
            }
        }

        if attackers >= 4 && unclaimer >= 1 {
            return;
        }

        if attackers < 4 && !should_spawn_unclaimer {
            let mut body = vec![Part::Move, Part::Move];
            let max_energy = room.energy_capacity_available();
            let mut cost = 100;

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

    let desired_work_parts = construction_sites / 3;
    if building_work_parts as usize >= desired_work_parts {
        return;
    }

    let body = crate::room::spawning::creep_sizing::builder(room, cache);
    let cost = get_body_cost(&body);

    spawn_manager.create_spawn_request(Role::Builder, body, 4.0, cost, None, None);

}

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

pub fn hauler(
    room: &Room,
    cache: &mut CachedRoom,
    spawn_manager: &mut SpawnManager,
    memory: &mut ScreepsMemory,
) {
    let priority = 5.0;

    let current_hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

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

    spawn_manager.create_spawn_request(Role::Hauler, body, priority, cost, None, None);
}

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

    spawn_manager.create_spawn_request(Role::FastFiller, body, 4.0, cost, None, None);
}

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

        let mut priority = 0.0;

        if miner_count < hauler_count {
            priority -= 1.0;
        }
        priority += ( parts_needed as f64 );

        priority = 500.0;

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
