use std::collections::HashMap;

use log::info;
use screeps::{
    game::get_object_by_id_typed, Creep, HasPosition, MaybeHasId, ObjectId, Position,
    RoomCoordinate, RoomXY,
};

use super::RoomCache;
use crate::traits::creep::CreepExtensions;

pub struct TrafficCache {
    pub move_targets: HashMap<ObjectId<Creep>, RoomXY>,
    pub move_requests: HashMap<ObjectId<Creep>, RoomXY>,
    pub movement_map: HashMap<RoomXY, ObjectId<Creep>>,
    pub visited_creeps: HashMap<ObjectId<Creep>, bool>,

    pub cached_ops: HashMap<ObjectId<Creep>, Vec<RoomXY>>,
    pub move_intents: u8,
}

impl TrafficCache {
    pub fn new() -> Self {
        Self {
            move_targets: HashMap::new(),
            move_requests: HashMap::new(),
            movement_map: HashMap::new(),
            visited_creeps: HashMap::new(),
            cached_ops: HashMap::new(),
            move_intents: 0,
        }
    }
}

pub struct TrafficProcs;

impl TrafficProcs {
    pub fn run_movement(room_cache: &mut RoomCache) {
        let mut creeps_with_movement: Vec<(ObjectId<Creep>, RoomXY)> = Vec::new();

        for creep in room_cache.creeps.creeps.values() {
            let Some(id) = creep.try_id() else {
                continue;
            };

            if let Some(creep_dest) = room_cache.traffic.move_requests.get(&id) {
                creeps_with_movement.push((id, *creep_dest));
            }
        }

        for (id, target) in creeps_with_movement {
            if room_cache.traffic.move_targets.get(&id)
                == room_cache.traffic.move_requests.get(&id)
            {
                continue;
            }

            room_cache.traffic.movement_map.remove(&target);
            room_cache.traffic.move_targets.remove(&id);

            let creep = get_object_by_id_typed(&id).unwrap();

            if creep.depth_first_searh(room_cache, None) > 0 {
                continue;
            }

            creep.assign_move_target(room_cache, target);
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
}
