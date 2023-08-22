use std::collections::HashMap;

use log::*;
use screeps::{
    find, game,
    prelude::*
};
use wasm_bindgen::prelude::*;

use crate::memory::ScreepsMemory;

mod building;
mod industries;
mod logging;
mod memory;
mod movement;
mod roles;
mod room;

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen(js_name = setup)]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    debug!("Loop starting! CPU: {}", game::cpu::get_used());
    let mut memory = ScreepsMemory::init_memory();

    industries::mining::pre_market(&mut memory);

    // This is done so infrequently for a few reasons:
    // 1. Im too lazy to figure out a way to iterate creeps in the main loop
    // 2. Because it doesnt use much decrease modulo as bot succeeds.
    // 3. Because fuck you thats why
    let mut detected_creeps: Vec<String> = Vec::new();
    if game::time() % 20 == 0 {
        for creep in game::creeps().keys() {
            match memory.creeps.get_mut(&creep) {
                Some(_) => {
                    detected_creeps.push(creep);
                },
                None => {
                    memory.create_creep(&creep, game::creeps().get(creep.clone()).unwrap().room().unwrap());
                    detected_creeps.push(creep);
                },
            }
        }
    }

    memory.creeps.retain(|x, _| detected_creeps.contains(x));

    room::spawning::run_spawns(&mut memory);

    // Bot is finished, write the local copy of memory.
    // This is run only once per tick as it serializes the memory.
    // This is done like this because its basically MemHack for you JS people.
    memory.write_memory();

    info!("Done! Cpu used: {}", game::cpu::get_used());
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
    memory.creeps = HashMap::new();
    memory.rooms = HashMap::new();
    memory.write_memory();
}
