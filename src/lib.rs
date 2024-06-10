use std::{collections::HashMap, sync::{Once, OnceLock}};

use combat::{ally::Allies, hate_handler::decay_hate};
use heap_cache::GlobalHeapCache;
use log::*;
use room::{cache::tick_cache::RoomCache, visuals::visualise_scouted_rooms};
use screeps::{find, game, OwnedStructureProperties, StructureProperties};
use wasm_bindgen::prelude::*;

use crate::{
    memory::ScreepsMemory,
    room::planning::{self, room::plan_room},
    traits::room::RoomExtensions,
};

mod combat;
mod config;
mod heap_cache;
mod logging;
mod memory;
mod movement;
mod room;
mod traits;
mod utils;

static INITIALIZED: Once = Once::new();

// pub static HEAP_CACHE: Lazy<GlobalHeapCache> = Lazy::new(GlobalHeapCache::new);
pub fn heap() -> &'static GlobalHeapCache {
    static HEAP: OnceLock<GlobalHeapCache> = OnceLock::new();
    HEAP.get_or_init(GlobalHeapCache::new)
}


#[wasm_bindgen]
pub fn init() {
    logging::setup_logging(LevelFilter::Info);
    info!("Initializing...");
}

#[wasm_bindgen]
// , screeps_timing_annotate::timing
//#[cfg(feature = "profile")]
pub fn game_loop() {
    use room::democracy;

    //#[cfg(feature = "profile")]
    {
        //screeps_timing::start_trace(Box::new(|| {
        //    (screeps::game::cpu::get_used() * 1000.0) as u64
        //}));
    }

    INITIALIZED.call_once(|| {
        init();
    });

    info!(
        "---------------- CURRENT TICK - {} ----------------",
        game::time()
    );

    if game::cpu::bucket() < 500 {
        info!("Bucket is too low, skipping tick");
        info!("Bucket: {}/500", game::cpu::bucket());
        return;
    }

    let mut memory = ScreepsMemory::init_memory();
    let mut cache = RoomCache::new();
    let mut allies = Allies::new(&mut memory);
    allies.sync(&mut memory);

    //if just_reset() {
    //
    //}

    for room in game::rooms().keys() {
        let game_room = game::rooms().get(room).unwrap();
        democracy::start_government(game_room, &mut memory, &mut cache);

    }

    // --- Start stats
    memory.stats.cpu.bucket = game::cpu::bucket();
    memory.stats.cpu.used = game::cpu::get_used();

    memory.stats.gcl_level = game::gcl::level();
    memory.stats.gcl_progress = game::gcl::progress();
    memory.stats.gcl_progress_total = game::gcl::progress_total();

    memory.stats.tick = game::time();
    memory.stats.credits = game::market::credits();

    memory.stats.age += 1;

    // --- End stats

    // Bot is finished, write the stats and local copy of memory.
    // This is run only once per tick as it serializes the memory.
    // This is done like this because its basically MemHack for you JS people.
    memory.write_memory();

    //#[cfg(feature = "profile")]
    {
        //let trace = screeps_timing::stop_trace();

        //if let Ok(trace_output) = serde_json::to_string(&trace) {
        //    info!("{}", trace_output);
        //}
    }

    decay_hate(&mut memory);
    visualise_scouted_rooms(&mut memory);

    let mut heap_lifetime = heap().heap_lifetime.lock().unwrap();

    let heap = game::cpu::get_heap_statistics();
    let used = (heap.total_heap_size() / heap.heap_size_limit()) * 100;

    info!("[STATS] Statistics are as follows: ");
    info!(
        "  GCL {}. Next: {} / {}",
        game::gcl::level(),
        game::gcl::progress(),
        game::gcl::progress_total()
    );
    info!("  CPU Usage:");
    info!("       Total: {}", game::cpu::get_used());
    info!("       Bucket: {}", game::cpu::bucket());
    info!("       Heap: {:.2}%", used);
    info!("       Heap Lifetime: {}", heap_lifetime);
    *heap_lifetime += 1;
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
    if game::time() == 0 {
        return true;
    }

    if game::creeps().entries().count() >= 1 {
        return false;
    }
    if game::rooms().entries().count() > 1 {
        return false;
    }

    let room = game::rooms().values().next().unwrap();

    if room.controller().is_none()
        || !room.controller().unwrap().my()
        || room.controller().unwrap().level() != 1
        || room.controller().unwrap().progress().unwrap() > 0
        || room.controller().unwrap().safe_mode().unwrap() > 0
    {
        return false;
    }

    if game::spawns().entries().count() != 1 {
        return false;
    }

    true
}
