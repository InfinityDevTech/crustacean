use std::collections::HashMap;

use log::*;
use screeps::{find, game, OwnedStructureProperties, StructureProperties};
use wasm_bindgen::prelude::*;

use crate::{memory::ScreepsMemory, room::planning::{self, room::plan_room}, traits::room::RoomExtensions};

mod logging;
mod memory;
mod utils;
mod movement;
mod room;
mod traits;
mod combat;

pub const MEMORY_VERSION: u8 = 1;

#[wasm_bindgen]
pub fn init() {
    logging::setup_logging(LevelFilter::Info);
    info!("Initializing...");
}

#[wasm_bindgen]
// , screeps_timing_annotate::timing
#[cfg(feature = "profile")]
pub fn game_loop() {
    #[cfg(feature = "profile")]
    {
        screeps_timing::start_trace(Box::new(|| {
            (screeps::game::cpu::get_used() * 1000.0) as u64
        }));
    }


    info!("---------------- CURRENT TICK - {} ----------------", game::time());


    let mut memory = ScreepsMemory::init_memory();

    if game::time() % 10 == 0 {
        for room in game::rooms().values() {
            if let Some(controller) = room.controller() {
                if controller.my() && !memory.rooms.contains_key(&room.name()) {
                    plan_room(&room, &mut memory);
                }
            }
        }
    }

    if just_reset() {
        for room in game::rooms().keys() {
            let room = game::rooms().get(room).unwrap();

            // If the planner says false on the first game tick, it doesnt have enough CPU to plan the room.
            // So we can fill teh bucket and try again next tick.
            if room.my() && !planning::room::plan_room(&room, &mut memory) { return; }
        }
    }
    
    for room in game::rooms().keys() {
        let game_room = game::rooms().get(room).unwrap();
        let room_memory = memory.rooms.get(&game_room.name());

        if room_memory.is_none() && game_room.my() { plan_room(&game_room, &mut memory); }

        room::democracy::start_government(game::rooms().get(room).unwrap(), &mut memory);
    }

    // Bot is finished, write the stats and local copy of memory.
    // This is run only once per tick as it serializes the memory.
    // This is done like this because its basically MemHack for you JS people.
    memory.write_memory();

    #[cfg(feature = "profile")]
    {
        let trace = screeps_timing::stop_trace();

        if let Ok(trace_output) = serde_json::to_string(&trace) {
            info!("{}", trace_output);
        }
    }

    let heap = game::cpu::get_heap_statistics();
    let used = (heap.total_heap_size() / heap.heap_size_limit()) * 100;

    info!("[STATS] Statistics are as follows: ");
    info!("  GCL {}. Next: {} / {}", game::gcl::level(), game::gcl::progress(), game::gcl::progress_total());
    info!("  CPU Usage:");
    info!("       Total: {}", game::cpu::get_used());
    info!("       Bucket: {}", game::cpu::bucket());
    info!("       Heap: {:.2}%", used);
}

#[wasm_bindgen(js_name = red_button)]
pub fn big_red_button() {
    for creep in game::creeps().values() {
        let _ = creep.say("WHY???", true);
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