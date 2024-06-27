use screeps::Room;

use crate::{memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

use super::setters;

// This portion is per-room, this is where most requests get made
// The actual execution is on the global scale
pub fn run_room_combat(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    setters::remote_reservation::determine_reservations(room, memory, cache);
}