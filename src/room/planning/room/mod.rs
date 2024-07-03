use log::info;
use screeps::{find, game, HasPosition, Room, RoomXY, StructureProperties};

use crate::{memory::{RoomMemory, ScreepsMemory}, room::cache::tick_cache::RoomCache, traits::room::RoomExtensions};

pub mod construction;
pub mod structure_visuals;
pub mod remotes;
pub mod roads;

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn plan_room(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    if game::cpu::bucket() < 500 {
        info!("  [PLANNER] CPU bucket is too low to plan room: {}", room.name_str());
        return false;
    }

    info!("  [PLANNER] Planning order recieved! Planning: {}", room.name_str());

    let my_spawn = room.find(find::MY_SPAWNS, None);
    let my_storage = room.find(find::MY_STRUCTURES, None).into_iter().filter(|s| s.structure_type() == screeps::StructureType::Storage).collect::<Vec<_>>();
    if my_spawn.is_empty() {
        info!("[PLANNER]  No spawns in room! Skipping planning!");
        return false;
    }

    let mut spawn = my_spawn.first().unwrap();

    let store_pos = if my_storage.is_empty() {
        let spawn_x = spawn.pos().x().u8();
        let spawn_y = spawn.pos().y().u8();

        unsafe { RoomXY::unchecked_new(spawn_x + 1, spawn_y + 3) }
    } else {
        let storage_x = my_storage.first().unwrap().pos().x().u8();
        let storage_y = my_storage.first().unwrap().pos().y().u8();

        unsafe { RoomXY::unchecked_new(storage_x + 1, storage_y - 1) }
    };

    let room_memory = RoomMemory {
        name: room.name(),
        rcl: room.controller().unwrap().level(),
        planned: false,
        id: 0,
        creeps: Vec::new(),
        remotes: Vec::new(),

        spawn_center: spawn.pos().xy(),
        storage_center: store_pos,

        hauler_count: 0,
        under_attack: false,
    };

    memory.create_room(&room.name(), room_memory);

    info!("[PLANNER]  Inserted room into memory! Making cache!");

    cache.create_if_not_exists(room, memory, None);
    true
}