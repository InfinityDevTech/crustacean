use log::info;
use screeps::{
    pathfinder::{self, SearchOptions},
    HasPosition, Position, Room, RoomCoordinate, RoomName,
};

use crate::{
    memory::{RemoteRoomMemory, ScreepsMemory},
    room::{cache::tick_cache::CachedRoom, democracy::remote_path_call},
    traits::room::{RoomExtensions, RoomType}, utils,
};

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn fetch_possible_remotes(
    room: &Room,
    memory: &mut ScreepsMemory,
    room_cache: &mut CachedRoom,
) -> Vec<RoomName> {
    // Little high on CPU, but its run every 3k ticks, so its fine. I guess.
    let mut pre_existing = Vec::new();
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

    info!("Possible remotes: {:?}", possible_remotes.len());

    let mut remotes = Vec::new();

    // Sort the remotes by rank - ascending
    possible_remotes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

    if possible_remotes.len() < 5 && !room_memory.remotes.is_empty() {
        return room_memory.remotes.clone();
    }

    // ONLY, wipe this if we find new remotes. Fix this later
    // TODO: Fix this so it doesnt remove existing ones for the tick duration. It fucks with things.
    if let Some(room_memory) = memory.rooms.get_mut(&room.name()) {
        for remote in room_memory.remotes.clone().iter() {
            memory.remote_rooms.remove(remote);
            pre_existing.push(*remote);

            room_memory.remotes.retain(|x| x != remote);
        }
    }

    if let Some(room_memory) = memory.rooms.get_mut(&room.name()) {
        // Get the top 2.
        for remote in possible_remotes.iter().take(5) {
            let remote = RemoteRoomMemory {
                name: remote.0,
                owner: room.name(),

                creeps: Vec::new(),
            };

            remotes.push(remote.name);
            room_memory.remotes.push(remote.name);
            memory.scouted_rooms.remove(&remote.name);
            memory.remote_rooms.insert(remote.name, remote);
        }
    }

    for remote in pre_existing {
        if remotes.contains(&remote) {
            continue;
        } else {
            let cloned = memory.goals.room_reservation.clone();
            let goal = cloned.values().filter(|x| x.reservation_target == remote);
            for goal in goal {
                memory.goals.room_reservation.remove(&goal.reservation_target);
            }
        }
    }

    remotes
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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
    if scouted.unwrap().owner.is_some() || scouted.unwrap().reserved.is_some() && *scouted.unwrap().reserved.as_ref().unwrap() != utils::get_my_username() {
        return u32::MAX;
    }

    if scouted.unwrap().room_type == RoomType::SourceKeeper
        || scouted.unwrap().room_type == RoomType::Highway
    {
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
