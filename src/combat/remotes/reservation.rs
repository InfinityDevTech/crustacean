use screeps::{game, HasPosition, OwnedStructureProperties, Position, Room, RoomCoordinate};

use crate::{goal_memory::RemoteReservationGoal, memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::position::PositionExtensions, utils::get_my_username};

pub fn determine_reservations(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    //let mut reservations = Vec::new();

    for remote in memory.remote_rooms.values() {
        let exists = memory.goals.remote_reservation.iter().any(|x| x.reservation_target == remote.name);
        if exists {
            continue;
        }

        let room = game::rooms().get(remote.name).unwrap();

        let reservation = does_remote_need_reservation(&room, memory, cache);

        let accessible_reservation_points = room.controller().unwrap().pos().get_accessible_positions_around(1);

        if reservation {
            let goal = RemoteReservationGoal {
                reservation_target: remote.name,
                accessible_reservation_spots: accessible_reservation_points,
                creeps_assigned: Vec::new(),
            };

            memory.goals.remote_reservation.push(goal);
        }
    }
}

pub fn does_remote_need_reservation(room: &Room, memory: &ScreepsMemory, cache: &RoomCache) -> bool {
    let remote_memory = memory.remote_rooms.get(&room.name()).unwrap();

    if room.controller().is_none() {
        return false;
    }

    let controller = room.controller().unwrap();
    if controller.reservation().is_none() {
        return true;
    }

    if controller.reservation().is_some() && controller.reservation().unwrap().username() != get_my_username() || controller.owner().is_some() {
        return false;
    }

    let reservation = controller.reservation().unwrap();

    let owning_room_cache = cache.rooms.get(&remote_memory.owner).unwrap();

    let twenty_five = RoomCoordinate::new(25).unwrap();
    let center_position = Position::new(twenty_five, twenty_five, room.name());
    let owner_center = owning_room_cache.structures.spawns.values().next().unwrap().pos();

    let distance = center_position.get_range_to(owner_center);

    if reservation.ticks_to_end() < distance || reservation.ticks_to_end() < 100 {
        return true;
    }

    false
}