#![allow(non_snake_case)]use std::collections::HashMap;

use screeps::{
    game, Creep, HasPosition, MaybeHasId, ObjectId, Position, RoomCoordinate, RoomXY, SharedCreepProperties
};

use super::{assign_creep_to_coordinate, CachedRoom};
use crate::{heap, memory::ScreepsMemory, traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking}};

pub fn solve_traffic_simple(creeps_with_movement_intent: &Vec<ObjectId<Creep>>, room_cache: &mut CachedRoom) {
    let mut visited_creeps = HashMap::new();

    for creep_id in creeps_with_movement_intent {
        let creep = game::get_object_by_id_typed(creep_id).unwrap();
        if room_cache.traffic.matched_coord.get(creep_id) == room_cache.traffic.intended_move.get(creep_id) {
            continue;
        }

        visited_creeps.clear();

        if room_cache.traffic.matched_coord.contains_key(creep_id) {
            room_cache.traffic.movement_map.remove(&room_cache.traffic.matched_coord[creep_id]);
        }
        room_cache.traffic.matched_coord.remove(creep_id);

        if depth_first_searh(&creep, room_cache, &mut visited_creeps, Some(0)) > 0 {
            continue;
        }

        assign_creep_to_coordinate(&creep, room_cache, creep.pos().xy());
    }
}

fn depth_first_searh(creep: &Creep, room_cache: &mut CachedRoom, visited_creeps: &mut HashMap<ObjectId<Creep>, bool>, score: Option<i32>) -> i32 {
    let mut score = score.unwrap_or(0);
    *visited_creeps.entry(creep.try_id().unwrap()).or_insert(true) = true;

    let possible_moves = creep.get_possible_moves_traffic(room_cache);

    let mut empty_tiles = Vec::new();
    let mut occupied_tiles = Vec::new();

    for coord in possible_moves.clone() {
        if room_cache.traffic.movement_map.contains_key(&coord) {
            occupied_tiles.push(coord);
        } else {
            empty_tiles.push(coord);
        }
    }

    for coord in possible_moves {
        if room_cache.traffic.intended_move.contains_key(&creep.try_id().unwrap()) && *room_cache.traffic.intended_move.get(&creep.try_id().unwrap()).unwrap() == coord {
            score += 1;
        }

        let occupying = room_cache.traffic.movement_map.get(&coord);

        if occupying.is_none() {
            if score > 0 {
                assign_creep_to_coordinate(creep, room_cache, coord)
            }
            return score;
        }

        if !visited_creeps.contains_key(occupying.unwrap()) || !visited_creeps.get(occupying.unwrap()).unwrap() {
            if room_cache.traffic.intended_move.contains_key(occupying.unwrap()) && *room_cache.traffic.intended_move.get(occupying.unwrap()).unwrap() == coord {
                score -= 1;
            }

            let result = depth_first_searh(&game::get_object_by_id_typed(occupying.unwrap()).unwrap(), room_cache, visited_creeps, Some(score));

            if result > 0 {
                assign_creep_to_coordinate(creep, room_cache, coord);
                return result;
            }
        }
    }

    i32::MIN
}