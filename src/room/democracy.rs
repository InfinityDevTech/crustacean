use log::info;
use screeps::{game, Room, RoomVisual};
use wasm_bindgen::convert::IntoWasmAbi;

use crate::{memory::ScreepsMemory, room::{cache::RoomCache, creeps::{local::hauler, organizer, recovery::recover_creeps}, planning::room::{construction::get_bunker_plan, structure_visuals::RoomVisualExt}, tower}};

use super::planning::creep::miner::formulate_miner;

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    info!("[GOVERNMENT] Starting government for room: {}", room.name());

    let mut cache = RoomCache::new_from_room(&room, memory);

    tower::run_towers(&room, &cache);

    let pre_creep = game::cpu::get_used();
    organizer::run_creeps(&room, memory, &mut cache);

    let creep_count = memory.creeps.len();
    info!("[GOVERNMENT] Creep CPU: {:.3} - Per Creep: {:.3}", game::cpu::get_used() - pre_creep, (game::cpu::get_used() - pre_creep) / creep_count as f64);

    let _ = formulate_miner(&room, memory, &mut cache);

    let pre_haul = game::cpu::get_used();
    cache.resources.create_haul_request_for_dropped_energy(&mut cache.hauling);

    for creep in cache.hauling.haulers.clone().iter() {
        let creep = game::creeps().get(creep.to_string()).unwrap();
        hauler::run_creep(&creep, memory, &mut cache);
    }
    info!("Hauler CPU: {:.3}", game::cpu::get_used() - pre_haul);

    let coords = (25, 19);
    let things = get_bunker_plan();

    let mut viz = RoomVisualExt::new(room.name());

    if game::time() % 10 == 0 {
        recover_creeps(memory);
    }

    for thing in things.iter() {
        let x_offset = thing.0 + coords.0;
        let y_offset = thing.1 + coords.1;
        viz.structure(x_offset.into(), y_offset.into(), thing.2, 0.5);
    }
}
