use log::info;
use screeps::{
    pathfinder::{self, SearchOptions},
    Position, Room, RoomCoordinate, RoomName,
};

use crate::{
    config, goal_memory::RemoteInvaderCleanup, memory::{RemoteRoomMemory, ScreepsMemory}, room::{cache::tick_cache::CachedRoom, democracy::remote_path_call}, traits::{position::RoomXYExtensions, room::{RoomExtensions, RoomType}}, utils
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn fetch_possible_remotes(
    room: &Room,
    memory: &mut ScreepsMemory,
    room_cache: &mut CachedRoom,
) -> Vec<RoomName> {
    // Little high on CPU, but its run every 3k ticks, so its fine. I guess.
    let mut pre_existing = Vec::new();
    let adjacent_rooms = room.get_adjacent(2);

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
    possible_remotes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    info!("  [REMOTES] Found {} possible remotes, picking...", possible_remotes.len());

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
        for (remote_name, score) in possible_remotes.iter().take(config::ROOM_REMOTE_COUNT.into()) {
            // I was too lazy to make it a string, so yk
            // u32::MAX -2 goes hard.
            if *score == u32::MAX - 2 {
                let goal = RemoteInvaderCleanup {
                    cleanup_target: *remote_name,
                    creeps_assigned: Vec::new(),
                };

                memory.goals.remote_invader_cleanup.insert(*remote_name, goal);

                // Continue as its un-usable, since its reserved.
                continue;
            }

            let sroom_memory = memory.scouted_rooms.get(remote_name).unwrap();

            let remote = RemoteRoomMemory {
                name: *remote_name,
                owner: room.name(),

                sources: sroom_memory.sources.as_ref().unwrap().to_vec(),

                creeps: Vec::new(),
                under_attack: false
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn rank_remote_room(
    memory: &mut ScreepsMemory,
    room_cache: &CachedRoom,
    remote_room: &RoomName,
) -> u32 {
    // If our room doesnt have a spawn placed yet.
    let spawn_pos = room_cache.spawn_center.unwrap().as_position(&room_cache.room_name);
    let mut i = 0;
    let mut current_avg = 0;

    // If we have no scouting data
    let scouted = memory.scouted_rooms.get(remote_room);
    // This >= 4 check is for SK rooms, idk why, or how, but my room classification is borked.
    if scouted.is_none() || scouted.unwrap().sources.is_none() || scouted.unwrap().sources.as_ref().unwrap().len() >= 3 {
        return u32::MAX;
    }

    // TODO: This should be changed to add aggression.
    // As of right now, we are pacificists.
    if scouted.unwrap().owner.is_some() || scouted.unwrap().reserved.is_some() && *scouted.unwrap().reserved.as_ref().unwrap() != utils::get_my_username() {
        if let Some(reservation) = scouted.unwrap().reserved.as_ref() {
            // FUCK these dues. Seriously, they are so FUCKING annoying.
            // They just delay my remotes, they are easy to delete, they just SUCK ASS.
            if reservation == "Invader" {
                return u32::MAX - 2;
            }
        }
        return u32::MAX;
    }

    if scouted.unwrap().room_type == RoomType::SourceKeeper
        || scouted.unwrap().room_type == RoomType::Highway
        || scouted.unwrap().room_type == RoomType::Center
    {
        return u32::MAX;
    }

    // Go thorugh each source and make a path to it, then average the cost.
    for source in scouted.as_ref().unwrap().sources.as_ref().unwrap() {
        let position = Position::new(
            RoomCoordinate::new(source.pos.x.u8()).unwrap(),
            RoomCoordinate::new(source.pos.y.u8()).unwrap(),
            *remote_room,
        );
        let options = Some(SearchOptions::new(remote_path_call).max_rooms(16).plain_cost(1).swamp_cost(5));
        let path = pathfinder::search(spawn_pos, position, 1, options);

        if path.incomplete() {
            return u32::MAX;
        }

        current_avg += path.cost();
        i += 1;
    }

    // We dont like one-source rooms, but if its
    // REALLY close, then we can pick it
    if i == 1 {
        current_avg += 15;
    }

    current_avg / i
}
