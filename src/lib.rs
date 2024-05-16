use std::{collections::HashMap, str::FromStr};

use log::*;
use screeps::{find, game, prelude::*, RoomName};
use wasm_bindgen::prelude::*;

use crate::{memory::ScreepsMemory, traits::room::RoomExtensions};

mod logging;
mod utils;
mod memory;
mod movement;
mod room;
mod visual;
mod traits;

pub const MEMORY_VERSION: u8 = 1;

#[wasm_bindgen(js_name = setup)]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

pub fn recently_respawned(memory: &mut ScreepsMemory) -> bool {
    if memory.spawn_tick || game::time() == 0 {
        return true;
    }

    let creeps = game::creeps().keys().collect::<Vec<String>>();
    if !creeps.is_empty() {
        return false;
    }

    let names: Vec<RoomName> = game::rooms().keys().collect();
    if names.len() != 1 {
        return false;
    }

    // check for controller, progress and safe mode
    let room = game::rooms().get(names[0]).expect("Failed to get room");
    let controller = room.controller();
    if controller.is_none()|| !controller.clone().unwrap().my() || controller.clone().unwrap().level() != 1 || controller.clone().unwrap().progress().unwrap() > 0 ||
       controller.clone().unwrap().safe_mode().is_none() {
        return false;
    }

    let spawns: Vec<String> = game::spawns().keys().collect();
    if spawns.len() != 1 {
        return false;
    }

    memory.spawn_tick = true;
    true
}

#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    info!("---------------- CURRENT TICK - {} ----------------", game::time());
    let before_memory = game::cpu::get_used();
    let mut memory = ScreepsMemory::init_memory();

    if game::time() % 10 == 0 {
        for room in game::rooms().values() {
            if let Some(controller) = room.controller() {
                if controller.my() && !memory.get_room(&room.name()).init {
                    memory.create_room(&room.name());
                }
            }
        }
    }

    if recently_respawned(&mut memory) {
        for room in game::rooms().keys() {
            let room = game::rooms().get(room).unwrap();
            if memory.rooms.get(&room.name_str()).is_some() {
                continue;
            }
            if let Some(controller) = room.controller() {
                if controller.my() {
                    memory.create_room(&room.name());
                }
            }
        }
        memory.spawn_tick = false
    }

    for room in memory.clone().rooms.values() {
        room::democracy::start_government(game::rooms().get(RoomName::from_str(&room.name).unwrap()).unwrap(), &mut memory);
    }

    visual::room::classify_rooms(&memory);

    // Bot is finished, write the stats and local copy of memory.
    // This is run only once per tick as it serializes the memory.
    // This is done like this because its basically MemHack for you JS people.
    memory.write_memory();

    info!("[DICTATOR] Government ran and memory written... Here are some stats!");
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
    memory.spawn_tick = true;
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
