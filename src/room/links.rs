use screeps::Room;

use crate::memory::ScreepsMemory;

use super::cache::tick_cache::{CachedRoom, RoomCache};

pub fn balance_links(room: &Room, room_cache: &mut CachedRoom) {
    //let room_cache = cache.rooms.get_mut(&room.name()).unwrap();
    if let Some(storage_link) = &room_cache.structures.links.storage {
        if let Some(controller_link) = &room_cache.structures.links.controller {
            let half_capacity = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy)) / 2;
            if controller_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy)) < half_capacity {
                let transfer_amount = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));
                let _ = storage_link.transfer_energy(controller_link, Some(transfer_amount));
            }
        }
    }
}