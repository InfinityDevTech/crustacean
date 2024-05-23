use log::info;
use screeps::{game, Room};

use crate::{memory::ScreepsMemory, room::{cache::RoomCache, creeps::{local::hauler, organizer}, tower}};

use super::planning::creep::miner::formulate_miner;

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    info!("[GOVERNMENT] Starting government for room: {}", room.name());
    let mut cache = RoomCache::new_from_room(&room, memory);

    let pre_creep = game::cpu::get_used();
    tower::run_towers(&room, &cache);
    organizer::run_creeps(&room, memory, &mut cache);
    let post_creep = game::cpu::get_used();

    let creep_count = memory.creeps.len();
    info!("[GOVERNMENT] Creep CPU: {:.3} - Per Creep: {:.3}", post_creep - pre_creep, (post_creep - pre_creep) / creep_count as f64);

    let _ = formulate_miner(&room, memory, &mut cache);

    cache.resources.create_haul_request_for_dropped_energy(&mut cache.hauling);

    for creep in cache.hauling.haulers.clone().iter() {
        let creep = game::creeps().get(creep.to_string()).unwrap();
        hauler::run_creep(&creep, memory, &mut cache);
    }
}
