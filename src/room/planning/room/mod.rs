use std::collections::HashMap;

use log::info;
use screeps::{find, game, pathfinder::SearchResults, HasPosition, Room, RoomXY, StructureProperties};

use crate::{memory::{RoomMemory, ScreepsMemory}, room::cache::tick_cache::RoomCache, traits::{intents_tracking::RoomExtensionsTracking, room::RoomExtensions}};

pub mod construction;
pub mod structure_visuals;
pub mod remotes;
pub mod roads;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn plan_room(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    if game::cpu::bucket() < 500 {
        info!("  [PLANNER] CPU bucket is too low to plan room: {}", room.name_str());
        return false;
    }

    info!("  [PLANNER] Planning order recieved! Planning: {}", room.name_str());

    let my_spawn = room.find(find::MY_SPAWNS, None);
    let my_storage = room.find(find::MY_STRUCTURES, None).into_iter().filter(|s| s.structure_type() == screeps::StructureType::Storage).collect::<Vec<_>>();
    let spawn_pos = if my_spawn.is_empty() {
        info!("[PLANNER]  No spawns in room! Attempting to use construction sites!");

        let spawn_csite = room.find(find::CONSTRUCTION_SITES, None).into_iter().filter(|s| s.structure_type() == screeps::StructureType::Spawn).collect::<Vec<_>>();

        if spawn_csite.is_empty() {
            info!("[PLANNER]  No spawn construction sites in room! Skipping planning!");
            return false;
        }

        unsafe { RoomXY::unchecked_new(spawn_csite.first().unwrap().pos().x().u8(), spawn_csite.first().unwrap().pos().y().u8() - 1) }
    } else {
        unsafe { RoomXY::unchecked_new(my_spawn.first().unwrap().pos().x().u8(), my_spawn.first().unwrap().pos().y().u8() - 1) }
    };

    let store_pos = if my_storage.is_empty() {
        let spawn_x = spawn_pos.x.u8();
        let spawn_y = spawn_pos.y.u8();

        unsafe { RoomXY::unchecked_new(spawn_x + 1, spawn_y + 3) }
    } else {
        let storage_x = my_storage.first().unwrap().pos().x().u8();
        let storage_y = my_storage.first().unwrap().pos().y().u8();

        unsafe { RoomXY::unchecked_new(storage_x + 1, storage_y - 1) }
    };

    let room_memory = RoomMemory {
        name: room.name(),
        max_rcl: room.controller().unwrap().level(),
        rcl: room.controller().unwrap().level(),
        planned: false,
        id: 0,
        creeps: Vec::new(),
        remotes: Vec::new(),

        rcl_times: HashMap::new(),

        spawn_center: spawn_pos,
        storage_center: store_pos,

        hauler_count: 0,
        under_attack: false,
    };

    memory.create_room(&room.name(), room_memory);

    info!("[PLANNER]  Inserted room into memory! Making cache!");

    cache.create_if_not_exists(room, memory, None);
    true
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn convert_path_to_roads(room: &Room, result: SearchResults) {
    if result.incomplete() {
        return;
    }

    let path = result.path();

    for pos in path {
        let x = pos.x().u8();
        let y = pos.y().u8();

        let _ = room.ITcreate_construction_site(x, y, screeps::StructureType::Road, None);
    }
}