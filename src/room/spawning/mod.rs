use log::info;
use screeps::Room;
use spawn_manager::SpawnManager;

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, utils::get_body_cost};

use super::cache::tick_cache::CachedRoom;

pub mod spawn_manager;
pub mod creep_sizing;

pub fn handle_spawning(room: &Room, room_cache: &mut CachedRoom, memory: &mut ScreepsMemory) {
    let mut spawn_manager = SpawnManager::new(&room.name(), room_cache);

    miner(room, room_cache, &mut spawn_manager);
    hauler(room, room_cache, &mut spawn_manager, memory);

    spawn_manager.run_spawning(room, room_cache, memory);
}

pub fn hauler(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager, memory: &mut ScreepsMemory) {
    let priority = 4.0;

    let current_hauler_count = cache.creeps.creeps_of_role.get(&Role::Hauler).unwrap_or(&Vec::new()).len();

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
        energy_in_room / 1000
    };

    if current_hauler_count as u32 >= haulers_to_make {
        return;
    }

    let body = crate::room::spawning::creep_sizing::hauler(room, cache);
    let cost = get_body_cost(&body);

    spawn_manager.create_spawn_request(Role::Hauler, body, priority, cost, None, None);
}

pub fn miner(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let miner_count = cache.creeps.creeps_of_role.get(&Role::Miner).unwrap_or(&Vec::new()).len();
    let hauler_count = cache.creeps.creeps_of_role.get(&Role::Hauler).unwrap_or(&Vec::new()).len();

    for source in &cache.resources.sources {
        let parts_needed = source.parts_needed();

        if parts_needed == 0 || source.creeps.len() >= source.calculate_mining_spots(room).into() {
            continue;
        }
        let parts = crate::room::spawning::creep_sizing::miner(room, cache, parts_needed);
        let cost = get_body_cost(&parts);

        let mut priority = 0.0;

        if miner_count < hauler_count { priority -= 1.0; }
        priority += parts_needed as f64;

        let index = &cache.resources.sources.iter().position(|s| s.id == source.id).unwrap();

        let creep_memory = CreepMemory {
            owning_room: room.name(),
            task_id: Some((*index).try_into().unwrap()),
            ..Default::default()
        };

        spawn_manager.create_spawn_request(Role::Miner, parts, priority, cost, Some(creep_memory), None);
    }
}