use std::collections::HashMap;

use log::info;
use screeps::{game, Room, SharedCreepProperties};

use crate::{
    combat::hate_handler::process_health_event, memory::{Role, ScreepsMemory}, room::{cache::{heap_cache::{HealthChangeType, HeapCreep}, tick_cache::RoomCache}, creeps::{global, remote}}, utils
};

use super::local;

pub fn run_creeps(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let pre_creeps_cpu = game::cpu::get_used();
    let mut cpu_usage_by_role = HashMap::new();
    let mut creeps_by_role = HashMap::new();

    // This is done in this manner to stop an "impossible" state
    // I reached, idk how, idk why, idk who, but it happened
    // and this is the only way I could think of to fix it
    let mut temp = cache.rooms.clone();
    let cached_room = temp.get_mut(&room.name()).unwrap();
    let creeps = &cached_room.creeps.creeps_in_room;

    let creeps = creeps.keys();

    if creeps.len() == 0 { return; }

    let starting_cpu = game::cpu::get_used();

    info!("  [CREEPS] Running {} creeps", creeps.len());
    let creep_count = creeps.len();

    let mut highest_user: String = "".to_string();
    let mut highest_usage: f64 = 0.0;

    for creep_name in creeps {
        let start_time = game::cpu::get_used();

        let creep = game::creeps().get(creep_name.clone()).unwrap();
        let mut role = Role::Recycler;

        if let Some(creep_memory) = memory.creeps.get(&creep.name()) {
            role = creep_memory.role;
        }

        if creep.spawning() { continue; }

        match role {
            Role::Miner => local::source_miner::run_creep(&creep, memory, cache),
            Role::Hauler => {
                info!("Hauler: {} - {}", creep.name(), room.name());
                cache.rooms.get_mut(&room.name()).unwrap().hauling.haulers.push(creep.name());
            }
            Role::Upgrader => local::upgrader::run_creep(&creep, memory, cache),
            Role::Builder => local::builder::run_creep(&creep, memory, cache),
            Role::FastFiller => local::fast_filler::run_creep(&creep, memory, cache),
            Role::Bulldozer => global::bulldozer::run_creep(&creep, memory, cache),
            Role::Scout => global::scout::run_creep(&creep, memory, cache),
            Role::GiftBasket => global::gift_drop::run_creep(&creep, memory, cache),
            Role::RemoteHarvester => remote::remote_harvester::run_creep(&creep, memory, cache),
            Role::Unclaimer => global::unclaimer::run_creep(&creep, memory, cache),

            Role::Recycler => global::recycler::run_creep(&creep, memory, cache),
        }

        let heap_creep = cached_room.heap_cache.creeps.entry(creep.name()).or_insert_with(|| HeapCreep::new(&creep));

        let health_change = heap_creep.get_health_change(&creep);
        if health_change != HealthChangeType::None {
            process_health_event(&creep, memory, health_change);
        }

        let end_time = game::cpu::get_used();
        let cpu_used = end_time - start_time;

        if cpu_used > highest_usage {
            highest_usage = cpu_used;
            highest_user = creep.name();
        }
        let end_time = game::cpu::get_used();

        if let Some(role) = cpu_usage_by_role.get_mut(&role) {
            *role += end_time - start_time;
        } else {
            cpu_usage_by_role.insert(role, end_time - start_time);
        }

        if let Some(role) = creeps_by_role.get_mut(&role) {
            *role += 1;
        } else {
            creeps_by_role.insert(role, 1);
        }
    }

    let room_cache = cache.rooms.get_mut(&room.name()).unwrap();
        room_cache.stats.creep_count = creep_count as u32;
        room_cache.stats.cpu_usage_by_role = cpu_usage_by_role;
        room_cache.stats.creeps_by_role = creeps_by_role;

    let end_cpu = game::cpu::get_used();
    info!("  [CREEPS] Used {:.4} CPU to run creeps {:.4} CPU per creep", end_cpu - starting_cpu, (end_cpu - starting_cpu) / creep_count as f64);
    info!("  [CREEPS] Highest CPU usage: {:.4} by {}", highest_usage, highest_user);

    if let Some(room) = cache.rooms.get_mut(&room.name()) {
        room.stats.cpu_creeps += game::cpu::get_used() - pre_creeps_cpu;
    }
}
