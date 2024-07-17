use screeps::{ResourceType, Room};

use crate::memory::ScreepsMemory;

use super::cache::{self, tick_cache::RoomCache};

pub fn run_terminal_stage1(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let room_cache = cache.rooms.get_mut(&room.name()).unwrap();
    let terminal_cache = &mut cache.terminals;
    // Determine energy needs
    if let Some(storage) = &room_cache.structures.storage {
        if storage.store().get_used_capacity(Some(ResourceType::Energy)) < 100000 {
            terminal_cache.mark_needs_resource(room.name(), vec![ResourceType::Energy, ResourceType::Battery])
        }
    }
}

pub fn process_terminal_transactions(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
}
