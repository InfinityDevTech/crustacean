use std::collections::HashMap;

use screeps::{game, Creep, HasPosition, MaybeHasId, ObjectId};

use crate::{room::cache::CachedRoom, traits::creep::CreepExtensions};

use super::assign_creep_to_coordinate;

pub fn solve_traffic_advanced(creeps_with_movement_intent: &Vec<ObjectId<Creep>>, room_cache: &mut CachedRoom) {
    let mut visited_creeps: HashMap<ObjectId<Creep>, bool> = HashMap::new();

    loop {
        let mut found = false;

        for creep_id in creeps_with_movement_intent {
            if room_cache.traffic.matched_coord.get(creep_id) == room_cache.traffic.intended_move.get(creep_id) {
                continue;
            }

            visited_creeps.clear();

            if room_cache.traffic.matched_coord.contains_key(creep_id) {
                room_cache.traffic.movement_map.remove(&room_cache.traffic.matched_coord[creep_id]);
            }
            room_cache.traffic.matched_coord.remove(creep_id);

            let creep = game::get_object_by_id_typed(creep_id).unwrap();

            if depth_first_search(&creep, room_cache, &mut visited_creeps, Some(0)) > 0 {
                found = true;
                continue;
            }

            assign_creep_to_coordinate(&creep, room_cache, creep.pos().xy());
        }

        if !found {
            break;
        }
    }
}

fn depth_first_search(creep: &Creep, room_cache: &mut CachedRoom, visited: &mut HashMap<ObjectId<Creep>, bool>, current_score: Option<i64>) -> i64 {
    let creep_id = creep.try_id().unwrap();
    *visited.entry(creep_id).or_insert(true) = true;

    let possible = creep.get_possible_moves_traffic(room_cache);

    for coord in possible {
        let mut score = current_score.unwrap_or(0);

        if room_cache.traffic.intended_move.contains_key(&creep_id) && room_cache.traffic.intended_move.get(&creep_id).unwrap() == &coord {
            score += 1;
        }

        let occupying = room_cache.traffic.movement_map.get(&coord);

        if occupying.is_none() {
            if score > 0 {
                assign_creep_to_coordinate(creep, room_cache, coord);
            }

            return score;
        }

        let occupying = *occupying.unwrap();
        if !visited.contains_key(&occupying) || !visited.get(&occupying).unwrap() {
            if room_cache.traffic.intended_move.contains_key(&occupying) && room_cache.traffic.intended_move.get(&occupying).unwrap() == &coord {
                score -= 1;
            }

            let result = depth_first_search(&game::get_object_by_id_typed(&occupying).unwrap(), room_cache, visited, Some(score));
            if result > 0 {
                assign_creep_to_coordinate(creep, room_cache, coord);
                return result;
            }
        }
    }

    i64::MIN
}