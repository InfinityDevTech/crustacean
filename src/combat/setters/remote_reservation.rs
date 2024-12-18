use screeps::{game, HasPosition, OwnedStructureProperties, Position, Room, RoomCoordinate};

use crate::{
    config,
    goal_memory::RoomReservationGoal,
    memory::ScreepsMemory,
    room::cache::RoomCache,
    traits::position::{PositionExtensions, RoomXYExtensions},
    utils::get_my_username,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn determine_reservations(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    for remote in memory.remote_rooms.values() {
        let exists = memory.goals.room_reservation.contains_key(&remote.name);
        if exists {
            continue;
        }

        let room = game::rooms().get(remote.name);

        if room.is_none() {
            continue;
        }
        let room = room.unwrap();

        // TODO: Make this spawn a dismantler, that way we can remove the wall
        // blocking it, and then claim it. I hate people that wall off controllers.
        let accessible_reservation_points = room
            .controller()
            .unwrap()
            .pos()
            .get_accessible_positions_around(1);
        if accessible_reservation_points.is_empty() {
            continue;
        }

        if remote_need_reservation(&room, memory, cache) {
            let goal = RoomReservationGoal {
                reservation_target: remote.name,
                accessible_reservation_spots: accessible_reservation_points.len() as u8,
                creeps_assigned: Vec::new(),
            };

            memory.goals.room_reservation.insert(room.name(), goal);
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn remote_need_reservation(room: &Room, memory: &ScreepsMemory, cache: &RoomCache) -> bool {
    let remote_memory = memory.remote_rooms.get(&room.name());
    if remote_memory.is_none() {
        return false;
    }

    let remote_memory = remote_memory.unwrap();

    if room.controller().is_none() {
        return false;
    }

    let controller = room.controller().unwrap();
    if controller.reservation().is_none() {
        return true;
    }

    if controller.reservation().is_some()
        && controller.reservation().unwrap().username() != get_my_username()
        || controller.owner().is_some()
    {
        return false;
    }

    let reservation = controller.reservation().unwrap();

    let owning_room_cache = cache.rooms.get(&remote_memory.owner);
    if let Some(owning_room_cache) = owning_room_cache {
        let twenty_five = RoomCoordinate::new(25).unwrap();
        let center_position = Position::new(twenty_five, twenty_five, room.name());
        let owner_center = owning_room_cache
            .spawn_center
            .unwrap()
            .as_position(&room.name());

        let distance = center_position.get_range_to(owner_center);

        if reservation.ticks_to_end() < distance
            || reservation.ticks_to_end() < config::RESERVATION_GOAL_THRESHOLD
        {
            return true;
        }
    }

    false
}
