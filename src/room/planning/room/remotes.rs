use log::info;
use screeps::{
    pathfinder::{self, SearchOptions}, HasPosition, MapTextStyle, Position, Room, RoomCoordinate, RoomName
};

use crate::{
    memory::ScreepsMemory,
    room::{cache::tick_cache::{CachedRoom, RoomCache}, democracy::remote_path_call},
    traits::room::RoomExtensions,
};

pub fn fetch_possible_remotes(
    room: &Room,
    memory: &mut ScreepsMemory,
    room_cache: &mut CachedRoom,
) -> Vec<RoomName> {
    let adjacent_rooms = room.get_adjacent(3);

    // Go through all the adjacent rooms and rank them
    let mut possible_remotes = Vec::new();

    for room_name in adjacent_rooms {
        let rank = rank_remote_room(memory, room_cache, &room_name);

        if rank == u32::MAX {
            continue;
        }

        possible_remotes.push((room_name, rank));
    }

    let mut remotes = Vec::new();

    // Sort the remotes by rank - ascending
    possible_remotes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());


    if let Some(room_memory) = memory.rooms.get_mut(&room.name()) {

    // Get the top 2.
    for remote in possible_remotes.iter().take(2) {
        remotes.push(remote.0);
        room_memory.remotes.push(remote.0);
    }
}

    remotes
}

pub fn rank_remote_room(
    memory: &mut ScreepsMemory,
    room_cache: &CachedRoom,
    remote_room: &RoomName,
) -> u32 {
    // If our room doesnt have a spawn placed yet.
    let spawn_pos = room_cache.structures.spawns.values().next();
    if spawn_pos.is_none() {
        return u32::MAX;
    }

    let mut i = 0;
    let mut current_avg = 0;

    // If we have no scouting data
    let scouted = memory.scouted_rooms.get(remote_room);
    if scouted.is_none() || scouted.unwrap().sources.is_none() {
        return u32::MAX;
    }

    // This should be changed to add aggression.
    // As of right now, we are pacificists.
    if scouted.unwrap().owner.is_some() || scouted.unwrap().reserved.is_some() {
        return u32::MAX;
    }

    // Go thorugh each source and make a path to it, then average the cost.
    for source in scouted.as_ref().unwrap().sources.as_ref().unwrap() {
        let position = Position::new(
            RoomCoordinate::new(source.x.u8()).unwrap(),
            RoomCoordinate::new(source.y.u8()).unwrap(),
            *remote_room,
        );
        let options = Some(SearchOptions::new(remote_path_call).max_rooms(16));
        let path = pathfinder::search(spawn_pos.unwrap().pos(), position, 1, options);

        current_avg += path.cost();
        i += 1;
    }

    // We dont like one-source rooms, but if its
    // REALLY close, then we can pick it
    if i == 1 {
        current_avg += 25;
    }

    current_avg / i
}
