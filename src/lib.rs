use std::{collections::HashMap, str::FromStr};

use log::*;
use screeps::{find, game, prelude::*, RoomName};
use wasm_bindgen::prelude::*;

use crate::{memory::ScreepsMemory, room::planning::{self, room::plan_room}, traits::room::RoomExtensions};

mod logging;
mod memory;
mod utils;
mod movement;
mod room;
mod traits;

pub const MEMORY_VERSION: u8 = 1;
static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    INIT_LOGGING.call_once(|| {
        // show all output of Info level, adjust as needed
        logging::setup_logging(logging::Info);
    });


    info!("---------------- CURRENT TICK - {} ----------------", game::time());


    let mut memory = ScreepsMemory::init_memory();

    if game::time() % 10 == 0 {
        for room in game::rooms().values() {
            if let Some(controller) = room.controller() {
                if controller.my() && !memory.rooms.contains_key(&room.name_str()) {
                    plan_room(&room, &mut memory);
                }
            }
        }

        for creep in memory.clone().creeps.keys() {
            if game::creeps().get(creep.clone()).is_none() {
                memory.creeps.remove(&creep.clone());
            }
        }
    }

    if just_reset() {
        for room in game::rooms().keys() {
            let room = game::rooms().get(room).unwrap();

            let controller = room.controller().unwrap();

            // If the planner says false on the first game tick, it doesnt have enough CPU to plan the room.
            // So we can fill teh bucket and try again next tick.
            if controller.my() && !planning::room::plan_room(&room, &mut memory) { return; }
        }
    }

    for room in memory.clone().rooms.values() {
        room::democracy::start_government(game::rooms().get(RoomName::from_str(&room.name).unwrap()).unwrap(), &mut memory);
    }

    // Bot is finished, write the stats and local copy of memory.
    // This is run only once per tick as it serializes the memory.
    // This is done like this because its basically MemHack for you JS people.
    memory.write_memory();

    let heap = game::cpu::get_heap_statistics();

    info!("[STATS] Statistics are as follows: ");
    info!("  GCL {}. Next: {} / {}", game::gcl::level(), game::gcl::progress(), game::gcl::progress_total());
    info!("  CPU Usage:");
    info!("       Total: {}", game::cpu::get_used());
    info!("       Bucket: {}", game::cpu::bucket());
    info!("       Heap: {:.1}/{:.1}", (heap.total_heap_size() / 1000000), (heap.heap_size_limit() / 1000000));
}

#[wasm_bindgen(js_name = red_button)]
pub fn big_red_button() {
    for creep in game::creeps().values() {
        let _ = creep.say("GOD WHY IS THIS HAPPENING???", true);
        let _ = creep.suicide();
    }
    for room in game::rooms().values() {
        if let Some(controller) = room.controller() {
            for structure in room.find(find::MY_STRUCTURES, None) {
                let _ = structure.destroy();
            }
            for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                let _ = csite.remove();
            }
            let _ = controller.unclaim();
        }
    }
    let mut memory = memory::ScreepsMemory::init_memory();
    memory.rooms = HashMap::new();
    memory.write_memory();
}

pub fn just_reset() -> bool {
    if game::time() == 0 { return true; }

    if game::creeps().entries().count() >= 1 { return false; }
    if game::rooms().entries().count() > 1 { return false; }

    let room = game::rooms().values().next().unwrap();

    if room.controller().is_none() || !room.controller().unwrap().my() || room.controller().unwrap().level() != 1 || room.controller().unwrap().progress().unwrap() > 0 || room.controller().unwrap().safe_mode().unwrap() > 0 {
        return false;
    }

    if game::spawns().entries().count() != 1 { return false; }

    true
}