use screeps::{
    pathfinder::{self, SearchOptions},
    HasPosition, Position, Room, RoomCoordinate, RoomName,
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
    if let Some(room_memory) = memory.rooms.get_mut(&room.name()) {
        if !room_memory.remotes.is_empty() {
            return room_memory.remotes.clone();
        }
    } else {
        return Vec::new();
    }

    let adjacent_rooms = room.get_adjacent();

    // Go through all the adjacent rooms and rank them
    let mut possible_remotes = Vec::new();
    for room_name in adjacent_rooms {
        let rank = rank_remote_room(memory, room_cache, &room_name);

        possible_remotes.push((room_name, rank));
    }

    let mut remotes = Vec::new();

    // Sort the remotes by rank - ascending
    possible_remotes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

    // Get the top 2.
    for remote in possible_remotes.iter().take(2) {
        remotes.push(remote.0);
        room_memory.remotes.push(remote.0);
    }

    remotes
}

pub fn rank_remote_room(
    memory: &mut ScreepsMemory,
    room_cache: &CachedRoom,
    remote_room: &RoomName,
) -> f32 {
    // If our room doesnt have a spawn placed yet.
    let spawn_pos = room_cache.structures.spawns.values().next();
    if spawn_pos.is_none() {
        return f32::MAX;
    }

    let mut i = 0.0;
    let mut current_avg = 0.0;

    // If we have no scouting data
    let scouted = memory.scouted_rooms.get(remote_room);
    if scouted.is_none() || scouted.unwrap().sources.is_none() {
        return f32::MAX;
    }

    // This should be changed to add aggression.
    // As of right now, we are pacificists.
    if scouted.unwrap().owner.is_some() || scouted.unwrap().reserved.is_some() {
        return f32::MAX;
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

        current_avg += path.cost() as f32;
        i += 1.0;
    }

    // We dont like one-source rooms, but if its
    // REALLY close, then we can pick it
    if i == 1.0 {
        current_avg += 25.0;
    }

    current_avg / i
}
