use screeps::Room;

use crate::{memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

use super::setters;

pub fn run_global_setters(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    setters::remote_reservation::determine_reservations(memory, cache);
    setters::remote_defense::determine_remote_defense_needs(cache, memory);
}