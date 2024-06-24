use std::{borrow::Borrow, collections::HashMap, sync::{Once, OnceLock}};

use combat::{ally::Allies, hate_handler::decay_hate};
use heap_cache::GlobalHeapCache;
use log::*;
use memory::{segment_ids, SegmentIDs};
use rand::{rngs::StdRng, Rng, SeedableRng};
use room::{cache::{self, tick_cache::{hauling, traffic, RoomCache}}, spawning, visuals::visualise_scouted_rooms};
use screeps::{find, game, IntershardResourceType, OwnedStructureProperties, StructureProperties};
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
mod constants;
mod utils;

static INITIALIZED: Once = Once::new();

// pub static HEAP_CACHE: Lazy<GlobalHeapCache> = Lazy::new(GlobalHeapCache::new);
pub fn heap() -> &'static GlobalHeapCache {
    static HEAP: OnceLock<GlobalHeapCache> = OnceLock::new();
    HEAP.get_or_init(GlobalHeapCache::new)
}

pub fn last_reset() -> &'static u32 {
    static LAST_RESET: OnceLock<u32> = OnceLock::new();
    LAST_RESET.get_or_init(game::time)
}


#[wasm_bindgen]
pub fn init() {
    logging::setup_logging(LevelFilter::Info);
    info!("Initializing...");
}

#[wasm_bindgen]
// , screeps_timing_annotate::timing
//#[cfg(feature = "profile")]

// TODO: Improve logistics, or improve remoting, either or
// Reserve remotes, we need the 3k energy from the sources.
// Fix the hauler reserving logic? It doesnt appear to be persistent cross tick.
// Either reserve more remotes, or add roads to the remotes.
pub fn game_loop() {
    use room::democracy;

    #[cfg(feature = "profile")]
    {
        screeps_timing::start_trace(Box::new(|| {
            (screeps::game::cpu::get_used() * 1000.0) as u64
        }));
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

        #[cfg(feature = "profile")]
        {
            finish_trace();
        }
    }

    let mut memory = heap().memory.lock().unwrap();
    let mut cache = RoomCache::new();
    let mut allies = Allies::new(&mut memory);
    allies.sync(&mut memory);

    memory.activate_segments();

    memory.stats.cpu.pathfinding = 0.0;

    //if just_reset() {
    //
    //}

    let pre_room_cpu = game::cpu::get_used();
    for room in game::rooms().keys() {
        let game_room = game::rooms().get(room).unwrap();
        democracy::start_government(game_room, &mut memory, &mut cache);
    }

    for room in cache.my_rooms.clone().iter() {
        let game_room = game::rooms().get(*room).unwrap();

        // Run spawns
        // TODO: Tune the better spawn implementation
        spawning::handle_spawning(&game_room, &mut cache, &mut memory);
        hauling::match_haulers(&mut cache, &mut memory, room);

        let room_cache = cache.rooms.get_mut(room).unwrap();

        // -- Begin creep chant stuffs
        let mut random = StdRng::seed_from_u64(game::time() as u64);
        let iterable = room_cache.creeps.creeps_in_room.values().collect::<Vec<_>>().to_vec();
        let random_creep = iterable[random.gen_range(0..room_cache.creeps.creeps_in_room.len())];

        let chant = config::CREEP_SONG;
        let chant_count = chant.len();

        let index = memory.chant_index;

        if index + 1 >= chant_count.try_into().unwrap() {
            memory.chant_index = 0;
        } else {
            memory.chant_index += 1;
        }

        let chant = chant[index as usize];
        let _ = random_creep.say(chant, true);
        // -- End creep chant stuffs
    }

    for room in game::rooms().keys() {
        let room = game::rooms().get(room).unwrap();
        if let Some(room_cache) = cache.rooms.get_mut(&room.name()) {
            traffic::run_movement(room_cache);

            if room.my() {
                room_cache.write_cache_to_heap(&room);
            }
        }
    }

    memory.stats.cpu.rooms = game::cpu::get_used() - pre_room_cpu;

    set_stats(&mut memory);

    // Bot is finished, write the stats and local copy of memory.
    // This is run only once per tick as it serializes the memory.
    // This is done like this because its basically MemHack for you JS people.
    if game::time() % 10 == 0 && game::cpu::bucket() > 3000 {
        info!("[MEMORY] Writing memory!");
        memory.write_memory();
    } else {
        info!("[MEMORY] Bucket is too low or tick isnt divisible by 10, skipping memory write");
    }

    decay_hate(&mut memory);

    if config::VISUALISE_SCOUTING_DATA {
        visualise_scouted_rooms(&mut memory);
    }

    let mut heap_lifetime = heap().heap_lifetime.lock().unwrap();

    let heap = game::cpu::get_heap_statistics();
    let used = ((heap.total_heap_size() as f64 + heap.externally_allocated_size() as f64) / heap.heap_size_limit() as f64) * 100.0;

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

    #[cfg(feature = "profile")]
    {
        let trace = screeps_timing::stop_trace();

        if let Ok(trace_output) = serde_json::to_string(&trace) {

            //info!("Trace output: {}", trace_output);
            let val = JsValue::from_str(&constants::COPY_TEXT.replace("$TO_COPY$", &trace_output).replace("$TIME", game::time().to_string().as_str()));
            web_sys::console::log_1(&val);
        }
    }
}

#[cfg(feature = "profile")]
pub fn finish_trace() {
    let trace = screeps_timing::stop_trace();

    if let Ok(trace_output) = serde_json::to_string(&trace) {
        screeps::raw_memory::segments().set(segment_ids()[SegmentIDs::Profiler], trace_output);
    }
}

#[cfg(feature = "profile")]
pub fn start_trace() {
    screeps_timing::start_trace(Box::new(|| {
        (screeps::game::cpu::get_used() * 1000.0) as u64
    }));
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_memory_usage_bytes() -> u32 {
    let memory = screeps::raw_memory::get().as_string().unwrap();
    let character_arr = memory.chars().collect::<Vec<char>>();

    character_arr.len() as u32
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn set_stats(memory: &mut ScreepsMemory) {
    let stats = &mut memory.stats;

    let market_resources = game::resources();
    let heap = game::cpu::get_heap_statistics();

    stats.tick = game::time();
    stats.last_reset = *last_reset();
    stats.age += 1;
    stats.gpl = game::gpl::level();

    stats.gcl.level = game::gcl::level();
    stats.gcl.progress = game::gcl::progress();
    stats.gcl.progress_total = game::gcl::progress_total();

    stats.market.credits = game::market::credits();
    //stats.market.cpu_unlocks = market_resources.get(IntershardResourceType::CpuUnlock);

    stats.memory_usage.total = 2 * 1000000;
    stats.memory_usage.used = get_memory_usage_bytes();

    stats.heap_usage.total = heap.heap_size_limit();
    stats.heap_usage.used = heap.total_heap_size() + heap.externally_allocated_size();

    stats.cpu.used = game::cpu::get_used();
    stats.cpu.bucket = game::cpu::bucket();
    stats.cpu.limit = game::cpu::limit();
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

#[wasm_bindgen(js_name = wipe_scouting_data)]
pub fn wipe_scouting_data() {
    let mut memory = heap().memory.lock().unwrap();
    memory.scouted_rooms = HashMap::new();
    memory.write_memory();
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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
