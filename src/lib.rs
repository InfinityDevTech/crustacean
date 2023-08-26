use std::{collections::HashMap, str::FromStr};

use log::*;
use screeps::{find, game, prelude::*, RoomName};
use wasm_bindgen::prelude::*;

use crate::memory::ScreepsMemory;

mod logging;
mod memory;
mod movement;
mod roles;
mod room;
mod visual;

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
    let room = game::rooms().get(names[0]).unwrap();
    let controller = room.controller();
    if controller.is_none()|| !controller.clone().unwrap().my() || controller.clone().unwrap().level() != 1 || controller.clone().unwrap().progress() > 0 ||
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
    let mut memory = ScreepsMemory::init_memory();

    if recently_respawned(&mut memory) {
        for room in game::rooms().keys() {
            let room = game::rooms().get(room).unwrap();
            if memory.rooms.get(&room.name().to_string()).is_some() {
                continue;
            }
            if let Some(controller) = room.controller() {
                if controller.my() {
                    memory.create_room(&room.name().to_string());
                }
            }
        }
        memory.spawn_tick = false
    }

    for room in memory.clone().rooms.values() {
        room::democracy::start_government(game::rooms().get(RoomName::from_str(&room.n).unwrap()).unwrap(), &mut memory);
    }

    visual::map::classify_rooms(&memory);

    // Bot is finished, write the local copy of memory.
    // This is run only once per tick as it serializes the memory.
    // This is done like this because its basically MemHack for you JS people.
    memory.write_memory();

    info!("[DICTATOR] Government ran and memory written... Here are some stats!");
    info!("CPU used: {}. Bucket: {}", game::cpu::get_used(), game::cpu::bucket());
    info!("GCL level {}. Next level: {} / {}", game::gcl::level(), game::gcl::progress(), game::gcl::progress_total());
    info!("Market credits: {}", game::market::credits());
}

#[wasm_bindgen(js_name = wipe_memory)]
pub fn wipe_memory() {
    let mut memory = ScreepsMemory::init_memory();
    memory.rooms = HashMap::new();
    memory.creeps = HashMap::new();
    memory.spawn_tick = true;
    memory.write_memory();
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
