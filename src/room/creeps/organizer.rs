use log::info;
use screeps::{game, Room, SharedCreepProperties};

use crate::{
    combat::hate_handler::process_health_event, memory::{Role, ScreepsMemory}, room::{cache::{heap_cache::{HealthChangeType, HeapCreep}, tick_cache::RoomCache}, creeps::global}, utils
};

use super::local;

pub fn run_creeps(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    // This is done in this manner to stop an "impossible" state
    // I reached, idk how, idk why, idk who, but it happened
    // and this is the only way I could think of to fix it
    let creeps = memory.rooms.get(&room.name());
    if creeps.is_none() {
        info!("  [CREEPS] No creeps in room {}", room.name());
        return;
    }
    let creeps = creeps.unwrap().creeps.clone();

    let starting_cpu = game::cpu::get_used();
    info!("  [CREEPS] Running {} creeps", creeps.len());
    let creep_count = creeps.len();

    let mut highest_user: String = "".to_string();
    let mut highest_usage: f64 = 0.0;

    for creep_name in creeps {
        let start_time = game::cpu::get_used();
        let creep = game::creeps().get(creep_name.to_string());

        if creep.is_none() {
            let _ = memory.creeps.remove(&creep_name);
            memory.rooms.get_mut(&room.name()).unwrap().creeps.retain(|x| x != &creep_name);
            continue;
        }

        let creep = creep.unwrap();
        let role = utils::name_to_role(&creep.name());

        if creep.spawning() || role.is_none() { return; }

        let heap_creep = cache.heap_cache.creeps.entry(creep.name()).or_insert_with(|| HeapCreep::new(&creep));

        let health_change = heap_creep.get_health_change(&creep);
        if health_change != HealthChangeType::None {
            process_health_event(&creep, memory, health_change);
        }

        match role.unwrap() {
            Role::Miner => local::source_miner::run_creep(&creep, memory, cache),
            Role::Hauler => {
                cache.hauling.haulers.push(creep.name());
            }
            Role::Upgrader => local::upgrader::run_creep(&creep, memory, cache),
            Role::Builder => local::builder::run_creep(&creep, memory, cache),
            Role::FastFiller => local::fast_filler::run_creep(&creep, memory, cache),
            Role::Bulldozer => local::bulldozer::run_creep(&creep, memory, cache),
            Role::Scout => global::scout::run_creep(&creep, memory, cache),
            _ => {}
        }

        let end_time = game::cpu::get_used();
        let cpu_used = end_time - start_time;

        if cpu_used > highest_usage {
            highest_usage = cpu_used;
            highest_user = creep.name();
        }
    }

    let end_cpu = game::cpu::get_used();
    info!("  [CREEPS] Used {:.4} CPU to run creeps {:.4} CPU per creep", end_cpu - starting_cpu, (end_cpu - starting_cpu) / creep_count as f64);
    info!("  [CREEPS] Highest CPU usage: {:.4} by {}", highest_usage, highest_user);
}
