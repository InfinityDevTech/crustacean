use screeps::Room;

use crate::{memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

use super::remotes;

pub fn run_room_combat(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    remotes::reservation::determine_reservations(room, memory, cache);
}