use crate::{memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

pub mod room_reservation;

pub fn run_goal_handlers(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    room_reservation::run_goal(memory, cache);
}