use std::{collections::HashMap, panic, str::FromStr};

use log::*;
use screeps::{find, game, prelude::*, RoomName};
use utils::utils::just_reset;
use wasm_bindgen::prelude::*;

use crate::{memory::ScreepsMemory, room::planning, traits::room::RoomExtensions};

mod logging;
mod utils;
mod memory;
mod movement;
mod room;
mod traits;

pub const MEMORY_VERSION: u8 = 1;

#[wasm_bindgen(js_name = setup)]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    info!("---------------- CURRENT TICK - {} ----------------", game::time());
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut memory = ScreepsMemory::init_memory();

    if game::time() % 10 == 0 {
        for room in game::rooms().values() {
            if let Some(controller) = room.controller() {
                if controller.my() && !memory.rooms.contains_key(&room.name_str()) {
                    memory.create_room(&room.name());
                }
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

    info!("[STATS] Statistics are as follows: ");
    info!("  GCL {}. Next: {} / {}", game::gcl::level(), game::gcl::progress(), game::gcl::progress_total());
    info!("  CPU Usage:");
    info!("       Total: {}", game::cpu::get_used());
    info!("       Bucket: {}", game::cpu::bucket());
}

#[wasm_bindgen(js_name = wipe_memory)]
pub fn wipe_memory() {
    let mut memory = ScreepsMemory::init_memory();
    memory.rooms = HashMap::new();
    memory.creeps = HashMap::new();
    memory.write_memory();
    info!("Memory wiped and written!");
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
