

#![allow(non_snake_case)]use std::collections::HashMap;

use log::info;
use screeps::{
    game::{self, get_object_by_id_typed}, Creep, Direction, HasPosition, MaybeHasId, ObjectId, Position, RoomCoordinate, RoomXY
};

use rand::prelude::SliceRandom;

use super::RoomCache;
use crate::{movement::utils::dir_to_coords, traits::creep::CreepExtensions};

pub struct TrafficCache {
    pub move_targets: HashMap<ObjectId<Creep>, RoomXY>,
    pub move_requests: HashMap<ObjectId<Creep>, RoomXY>,
    pub movement_map: HashMap<RoomXY, ObjectId<Creep>>,

    pub cached_ops: HashMap<ObjectId<Creep>, Vec<RoomXY>>,
    pub move_intents: u8,
}

impl TrafficCache {
    pub fn new() -> Self {
        Self {
            move_targets: HashMap::new(),
            move_requests: HashMap::new(),
            movement_map: HashMap::new(),
            cached_ops: HashMap::new(),
            move_intents: 0,
        }
    }
}

pub fn run_movement(room_cache: &mut RoomCache) {
    let mut creeps_with_movement: Vec<(ObjectId<Creep>, RoomXY)> = Vec::new();

    let creep_names: Vec<String> = room_cache.creeps.creeps.keys().cloned().collect();
    for creep_name in creep_names {
        let creep = game::creeps().get(creep_name.to_string()).unwrap();
        let Some(id) = creep.try_id() else {
            continue;
        };

        assign_move_target(&creep, room_cache, creep.pos().xy());

        if let Some(creep_dest) = room_cache.traffic.move_requests.get(&id) {
            creeps_with_movement.push((id, *creep_dest));
        }
    }

    let mut visited_creeps = Vec::new();

    for (id, target) in creeps_with_movement {
        //visited_creeps.clear();
        if room_cache.traffic.move_targets.get(&id)
            == room_cache.traffic.move_requests.get(&id)
        {
            continue;
        }

        room_cache.traffic.movement_map.remove(&target);
        room_cache.traffic.move_targets.remove(&id);

        let creep = get_object_by_id_typed(&id).unwrap();

        if depth_first_searh(&creep, room_cache, &mut visited_creeps, None) > 0 {
            continue;
        }

        assign_move_target(&creep, room_cache, target);
    }

    for creep in room_cache.creeps.creeps.values() {
        let matched_move = room_cache
            .traffic
            .move_targets
            .get(&creep.try_id().unwrap());
        if matched_move.is_none() {
            continue;
        }

        if &creep.pos().xy() == matched_move.unwrap() {
            continue;
        }

        let mat = matched_move.unwrap();

        let dir = creep
            .pos()
            .get_direction_to(Position::new(
                RoomCoordinate::new(mat.x.u8()).unwrap(),
                RoomCoordinate::new(mat.y.u8()).unwrap(),
                creep.room().unwrap().name(),
            ))
            .unwrap();

        let move_res = creep.move_direction(dir);
        if move_res.is_ok() {
            room_cache.traffic.move_intents += 1;
        } else {
            info!("Creep move failed, {:?}", move_res.err().unwrap());
        }
    }
}

fn depth_first_searh(creep: &Creep, room_cache: &mut RoomCache, visited_creeps: &mut Vec<ObjectId<Creep>>, score: Option<i32>) -> i32 {
    let id = creep.try_id();

    visited_creeps.push(id.unwrap());

    for roomxy in creep.get_possible_moves(room_cache) {
        let mut score = score.unwrap_or(0);

        if room_cache.traffic.move_requests.get(&id.unwrap()) == Some(&roomxy) {
            score += 1;
        }

        let Some(occupying_creep) = room_cache.traffic.movement_map.get(&roomxy) else {
            if score > 0 {
                assign_move_target(creep, room_cache, roomxy);
            }
            return score;
        };

        if !visited_creeps.contains(occupying_creep)
        {
            if room_cache
                .traffic
                .move_requests
                .get(occupying_creep)
                .unwrap()
                == &roomxy
            {
                score -= 1;
            }

            let result = depth_first_searh(creep, room_cache, visited_creeps, Some(score));

            if result > 0 {
                assign_move_target(creep, room_cache, roomxy);
                return result;
            }
        }
    }

    i32::MIN
}

fn assign_move_target(creep: &Creep, room_cache: &mut RoomCache, coord: RoomXY) {
    let id = creep.try_id();
    if id.is_none() {
        return;
    }

    room_cache.traffic.move_targets.insert(id.unwrap(), coord);
    room_cache.traffic.movement_map.insert(coord, id.unwrap());
}