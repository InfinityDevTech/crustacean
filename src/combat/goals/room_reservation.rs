use crate::{goal_memory::RoomReservationGoal, memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

pub fn attain_reservations(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let reservation_goals = memory.goals.room_reservation.iter_mut();

    for goal in reservation_goals {

    }
}

pub fn spawn_creep(goal: &mut RoomReservationGoal, memory: &mut ScreepsMemory, cache: &mut RoomCache) {

}