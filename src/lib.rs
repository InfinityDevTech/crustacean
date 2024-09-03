#![allow(internal_features)]
#![feature(map_many_mut)]
#![feature(core_intrinsics)]
#![feature(const_refs_to_static)]

use std::{
    collections::HashMap,
    sync::{Mutex, Once, OnceLock},
};

use alloc::ALLOCATOR;
use combat::{ally::Allies, global::run_global_goal_setters, goals::run_goal_handlers, hate_handler::decay_hate};
use constants::{MAX_BUCKET, MMO_SHARD_NAMES};
use formation::formations::run_formations;
use heap_cache::GlobalHeapCache;
use log::*;
use memory::Role;
use movement::caching::generate_pathing_targets;
use profiling::timing::{INTENTS_USED, SUBTRACT_INTENTS};
use rand::{rngs::StdRng, Rng, SeedableRng};
use room::{
    cache::{self, hauling, traffic, RoomCache}, democracy::start_government, expansion::{attempt_expansion, can_expand}, spawning::spawn_manager::{self, run_spawning, SpawnManager}, visuals::visualise_scouted_rooms
};
use screeps::{find, game, OwnedStructureProperties};
use traits::{creep::CreepExtensions, intents_tracking::{
    ConstructionExtensionsTracking, CreepExtensionsTracking, StructureControllerExtensionsTracking,
    StructureObjectTracking,
}};
use wasm_bindgen::prelude::*;

use crate::{memory::ScreepsMemory, traits::room::RoomExtensions};

mod alloc;
mod allies;
mod combat;
mod config;
mod constants;
mod goal_memory;
mod heap_cache;
mod logging;
mod memory;
mod movement;
mod profiling;
mod room;
mod traits;
mod formation;
mod utils;
mod compression;

static INITIALIZED: Once = Once::new();
pub static CLEAN_PROFILE: Mutex<bool> = Mutex::new(true);

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

pub fn game_loop() {
    #[cfg(feature = "profile")]
    {
        if game::cpu::bucket() > 200 {
            profiling::timing::start_trace(Box::new(|| {
                (screeps::game::cpu::get_used() * 1000.0) as u64
            }));
        }
    }

    INITIALIZED.call_once(|| {
        init();
    });

    if !config::USERNAME_LOCK.contains(&utils::get_my_username().to_lowercase().as_str()) {
        for _ in 0..10 {
            info!("");
        }

        info!("Hello, whoever you are. I am not for you.");
        info!("If you have acquired a copy of this code, please do not use it.");
        info!("This is a private project, and I do not want it to be used by others.");
        info!("If you are interested in the project, please contact me. I am open to talking.");
        info!("DM me on discord: inf5");
        return;
    }

    info!(
        "---------------- CURRENT TICK - {} ----------------",
        game::time()
    );

    if game::cpu::bucket() < 500 {
        info!("Bucket needed for profiling: {}/500", game::cpu::bucket());

        #[cfg(feature = "profile")]
        {
            let _ = crate::profiling::timing::stop_trace();
        }
    }

    let mut memory = heap().memory.lock().unwrap();
    let spawn_manager = SpawnManager::new();
    let mut cache = RoomCache::new(spawn_manager);
    let mut allies = Allies::new(&mut memory);
    allies.sync(&mut memory);

    memory.activate_segments();

    {
        let mut csay = heap().creep_say.lock().unwrap();
        *csay = memory.creep_say;

        *SUBTRACT_INTENTS.lock().unwrap() = memory.subtract_intents_profiler;
    }

    memory.stats.cpu.pathfinding = 0.0;

    let pre_room_cpu = game::cpu::get_used();
    for room in game::rooms().keys() {
        let game_room = game::rooms().get(room).unwrap();
        if game::cpu::bucket() < 100 && game::cpu::get_used() > game::cpu::limit() as f64 * 0.5 {
            continue;
        }

        start_government(game_room, &mut memory, &mut cache);
    }

    if game::time() % 1500 == 0 {
        for room in memory.rooms.clone().keys() {
            let groom = game::rooms().get(*room);

            if groom.is_none() {
                let old_room = memory.rooms.remove(room);

                if let Some(old_room) = old_room {
                    for remote in old_room.remotes {
                        memory.remote_rooms.remove(&remote);
                    }
                }
            }
        }
    }

    let pre_haul_cpu = game::cpu::get_used();
    for room in cache.my_rooms.clone().iter() {
        hauling::match_haulers(&mut cache, &mut memory, room);

        spawn_manager::calculate_hauler_needs(
            &game::rooms().get(*room).unwrap(),
            &mut memory,
            &mut cache,
        );

        if !cache.rooms.contains_key(room) || !memory.rooms.contains_key(room) {
            continue;
        }

        let room_cache = cache.rooms.get_mut(room).unwrap();
        let room_memory = memory.rooms.get_mut(room).unwrap();

        // -- Begin creep chant stuffs
        if !room_cache.creeps.creeps_in_room.is_empty() {
            let mut random = StdRng::seed_from_u64(game::time() as u64);
            let iterable = room_cache
                .creeps
                .creeps_in_room
                .values()
                .collect::<Vec<_>>()
                .to_vec();
            let random_creep =
                iterable[random.gen_range(0..room_cache.creeps.creeps_in_room.len())];

            let chant = config::CREEP_SONG;
            let chant_count = chant.len();

            let index = room_memory.chant_index;

            if index + 1 >= chant_count.try_into().unwrap() {
                room_memory.chant_index = 0;
            } else {
                room_memory.chant_index += 1;
            }

            let chant = chant[index as usize];
            random_creep.bsay(chant, true);
            // -- End creep chant stuffs
        }
    }
    info!("[HAULING] Government wide hauling took {:.2} CPU.", game::cpu::get_used() - pre_haul_cpu);

    run_global_goal_setters(&mut memory, &mut cache);
    run_goal_handlers(&mut memory, &mut cache);
    run_formations(&mut memory, &mut cache);

    if game::cpu::bucket() > 100 {
        run_spawning(&mut memory, &mut cache);
    }

    if game::time() % 100 == 0 {
        memory.filter_old_creeps();

        hauling::clean_heap_hauling(&mut memory);
    }

    let pre_traffic_cpu = game::cpu::get_used();
    let mut intent_count = 0;

    for room in game::rooms().keys() {
        let room = game::rooms().get(room).unwrap();
        if let Some(room_cache) = cache.rooms.get_mut(&room.name()) {
            let start = game::cpu::get_used();
            intent_count += traffic::run_movement(room_cache, &mut memory);

            if room.my() {
                info!(
                    "[TRAFFIC] {} Rooms traffic took: {:.4} with {} intents, {:.4} without intents",
                    room.name().to_string(),
                    game::cpu::get_used() - start,
                    room_cache.traffic.move_intents,
                    game::cpu::get_used() - start - (room_cache.traffic.move_intents as f64 * 0.2)
                );
            }

            if room.my() {
                room_cache.write_cache_to_heap(&room);
            }
        }
    }

    let traffic_cpu = game::cpu::get_used() - pre_traffic_cpu;
    info!("[TRAFFIC] Government wide traffic took {:.2} CPU. Without the {} intents {:.2}", traffic_cpu, intent_count, traffic_cpu - (intent_count as f64 * 0.2));

    if game::time() % 10000 == 0 {
        heap().cachable_positions.lock().unwrap().clear();
        heap().flow_cache.lock().unwrap().clear();
    }

    for room in heap().needs_cachable_position_generation.lock().unwrap().iter() {
        if let Some(room) = game::rooms().get(*room) {
            if memory.rooms.contains_key(&room.name()) || memory.remote_rooms.contains_key(&room.name()) {
                if let Some(room_cache) = cache.rooms.get_mut(&room.name()) {
                    generate_pathing_targets(&room, &memory, room_cache);
                }
            }
        }
    }

    memory.stats.cpu.rooms = game::cpu::get_used() - pre_room_cpu;

    set_stats(&mut memory);
    decay_hate(&mut memory);

    if game::flags().get("reset_expansion".to_string()).is_some() {
        memory.goals.room_claim.clear();
        memory.expansion = None;
    }

    if game::cpu::bucket() > 2000 && can_expand(&memory) {
        attempt_expansion(&mut memory, &cache);
    } else if game::cpu::bucket() < 2000 {
        info!("[EXPANSION] Not enough CPU to run! Waiting for 2k in the bucket!");
    } else if memory.expansion.is_some() {
        memory.expansion = None;
    }

    // Bot is finished, write the stats and local copy of memory.
    // This is run only once per tick as it serializes the memory.
    // This is done like this because its basically MemHack for you JS people.
    if (game::time() % 10 == 0 && game::cpu::bucket() > 3000 && game::cpu::get_used() < 300.0)
        || game::time() % 50 == 0
    {
        info!("[MEMORY] Writing memory!");
        memory.write_memory();
    } else {
        info!("[MEMORY] Bucket is too low, CPU usage is too high, or tick isnt divisible by 10, skipping memory write");
    }

    if config::VISUALISE_SCOUTING_DATA {
        visualise_scouted_rooms(&mut memory);
    }

    let mut heap_lifetime = heap().heap_lifetime.lock().unwrap();
    let intents_used = *INTENTS_USED.lock().unwrap();
    heap().per_tick_cost_matrixes.lock().unwrap().clear();
    heap().needs_cachable_position_generation.lock().unwrap().clear();
    run_creep_says();
    *INTENTS_USED.lock().unwrap() = 0;

    let heap = game::cpu::get_heap_statistics();
    let used = ((heap.total_heap_size() as f64 + heap.externally_allocated_size() as f64)
        / heap.heap_size_limit() as f64)
        * 100.0;

    let cpu_usage_percent = (game::cpu::get_used() as f32 / game::cpu::limit() as f32) * 100.0;

    let percentage_to_next_gcl = (game::gcl::progress() / game::gcl::progress_total()) * 100.0;

    info!(
        "GCL {}. {:.2}% to level {}",
        game::gcl::level(),
        percentage_to_next_gcl,
        game::gcl::level() + 1,
    );
    info!(
        "Used {:.2}% CPU. Used {:.2}% without intents.",
        cpu_usage_percent,
        game::cpu::get_used() - (intents_used as f64 * 0.2)
    );
    info!(
        "  Total: {:.4} - {} intents using {:.1} CPU. CPU without intents: {:.4}",
        game::cpu::get_used(),
        intents_used,
        intents_used as f32 * 0.2,
        game::cpu::get_used() - (intents_used as f64 * 0.2)
    );
    let mut highest_cpu_user = None;
    let mut highest = 0.0;
    let mut total_highest = 0.0;
    let mut total_highest_count = 0;

    for (role, role_used) in cache.creep_cpu_by_role {
        let role_count = *cache.creep_count_by_role.get(&role).unwrap_or(&0) as f64;

        if role == Role::BaseHauler {
            continue;
        }

        if (role_used / role_count) > highest {
            highest_cpu_user = Some(role);
            highest = role_used / role_count;
            total_highest = role_used;
            total_highest_count = role_count as u32;
        }
    }

    info!(
        "  {} non-owned rooms took {:.2} CPU. {} creeps took {:.2} CPU. - Highest role: {:?} with a whopping {:.2} AVG ({:.2} CPU across {} creeps)",
        cache.non_owned_count,
        cache.non_owned_cpu,
        cache.creep_count,
        cache.creep_cpu,
        highest_cpu_user.unwrap_or(Role::Recycler),
        highest,
        total_highest,
        total_highest_count
    );
    info!("  Bucket: {}", game::cpu::bucket());
    info!("  Heap: {:.2}% ({:.2} mb)", used, ((heap.total_heap_size() as f64 + heap.externally_allocated_size() as f64) / 1024.0 / 1024.0));
    info!("  Allocated now: {:?} bytes", ALLOCATOR.allocated_now());
    info!("  Allocated now: {:.2} mb", (ALLOCATOR.allocated_now() as f64 / 1024.0 / 1024.0));
    info!("  Time since last reset: {}", heap_lifetime);
    *heap_lifetime += 1;

    // TODO:
    // Make it so we check if we arent in combat either, or we arent going to do anything
    // High CPU, (like base building) then we can generate pixels.
    if MMO_SHARD_NAMES.contains(&game::shard::name().as_str()) {
        let cpu_usage = game::cpu::get_used();
        let bucket = game::cpu::bucket();

        if cpu_usage < 500.0 && bucket == MAX_BUCKET {
            info!("[PIXELS] We have enough CPU, generating pixel!");
            let _ = game::cpu::generate_pixel();
        }
    }

    /*if game::cpu::bucket() > 1000 {
    let origin = Position::new(RoomCoordinate::new(37).unwrap(), RoomCoordinate::new(16).unwrap(), RoomName::new("W1N9").unwrap());
        let dest = Position::new(RoomCoordinate::new(11).unwrap(), RoomCoordinate::new(17).unwrap(), RoomName::new("W1N9").unwrap());

        let o = origin.clone();
        let mut m = memory.clone();

        let callback = Box::new(move |r: RoomName| {
            crate::movement::move_target::lcl_call(r, origin, &m, MoveOptions::default())
        });

        let mut call = PathFinder::setup(origin, vec![dest], callback, 1, 5, 1, 10000, u32::MAX, false, 1.2);

        let res = call.search();

        if !res.incomplete {
            info!("AAAAAAAAAAAAAAAAAAAAAAAA");
            info!("Successfull: {:?}", res);
            visualise_path(res.path, origin, "#ff0000");
        } else {
            info!("AAAAAAAAAAAAAAAAAAAAAAA");
            info!("Failed!");
        }
    }*/

    #[cfg(feature = "profile")]
    {
        if game::cpu::bucket() > 200 {
            let trace = profiling::timing::stop_trace();

            if let Ok(trace_output) = serde_json::to_string(&trace) {
                //info!("Trace output: {}", trace_output);
                let val = JsValue::from_str(
                    &constants::COPY_TEXT
                        .replace("$TO_COPY$", &trace_output)
                        .replace("$TIME", game::time().to_string().as_str()),
                );
                web_sys::console::log_1(&val);
            }
        }
    }
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

    let heap = game::cpu::get_heap_statistics();
    let resources = game::resources();

    stats.tick = game::time();
    stats.last_reset = *last_reset();
    stats.age += 1;
    stats.gpl = game::gpl::level();

    stats.gcl.level = game::gcl::level();
    stats.gcl.progress = game::gcl::progress();
    stats.gcl.progress_total = game::gcl::progress_total();

    stats.market.credits = game::market::credits();
    stats.market.cpu_unlocks = resources
        .get(screeps::IntershardResourceType::CpuUnlock)
        .unwrap_or(0);
    stats.market.access_keys = resources
        .get(screeps::IntershardResourceType::AccessKey)
        .unwrap_or(0);
    stats.market.pixels = resources
        .get(screeps::IntershardResourceType::Pixel)
        .unwrap_or(0);

    stats.memory_usage.total = 2 * 1000000;
    stats.memory_usage.used = get_memory_usage_bytes();

    stats.heap_usage.total = heap.heap_size_limit();
    stats.heap_usage.used = heap.total_heap_size() + heap.externally_allocated_size();

    stats.cpu.used = game::cpu::get_used();
    stats.cpu.bucket = game::cpu::bucket();
    stats.cpu.limit = game::cpu::limit();
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn run_creep_says() {
    let do_says = heap().creep_say.lock().unwrap();

    if *do_says {
        let says = heap().per_tick_creep_says.lock().unwrap();

        for (creep_name, (public, say)) in says.clone() {
            if let Some(game_creep) = game::creeps().get(creep_name) {
                let _ = game_creep.say(&say, public);
            }
        }
    }

    heap().per_tick_creep_says.lock().unwrap().clear();
}

#[wasm_bindgen(js_name = red_button)]
pub fn big_red_button() {
    for creep in game::creeps().values() {
        let _ = creep.say("WHY???", true);
        let _ = creep.ITsuicide();
    }
    for room in game::rooms().values() {
        if let Some(controller) = room.controller() {
            for structure in room.find(find::MY_STRUCTURES, None) {
                let _ = structure.ITdestroy();
            }
            for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                let _ = csite.ITremove();
            }
            let _ = controller.ITunclaim();
        }
    }

    let mut memory = memory::ScreepsMemory::init_memory();
    memory.rooms = HashMap::new();
    memory.write_memory();
}

#[wasm_bindgen(js_name = toggle_creepsay)]
pub fn toggle_creepsay() {
    let mut heap_mem = heap().memory.lock().unwrap();

    heap_mem.creep_say = !heap_mem.creep_say;
}

#[wasm_bindgen(js_name = toggle_intent_subtraction)]
pub fn toggle_intent_subtraction() {
    let mut heap_mem = heap().memory.lock().unwrap();

    heap_mem.subtract_intents_profiler = !heap_mem.subtract_intents_profiler;
}

#[wasm_bindgen(js_name = wipe_memory)]
pub fn wipe_memory() {
    let mut heap_mem = heap().memory.lock().unwrap();

    let mut new_mem = ScreepsMemory::init_memory();

    new_mem.write_memory();
    *heap_mem = new_mem;
}

#[wasm_bindgen(js_name = hauler_rescan)]
pub fn manual_hauler_rescan() {
    let mut memory = heap().memory.lock().unwrap();
    for rmemory in &mut memory.rooms.values_mut() {
        rmemory.hauler_count = 0;
    }
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
