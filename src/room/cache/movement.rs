use std::collections::HashMap;

use log::info;
use screeps::{game, Creep, Direction, HasPosition, MaybeHasId, ObjectId, Position, Room, RoomCoordinate, RoomXY};

use crate::{movement::utils::dir_to_coords, traits::creep::CreepExtensions};

use rand::thread_rng;
use rand::seq::SliceRandom;

use super::RoomCache;

#[derive(Debug, Clone)]
pub struct RoomMovementCache {
    pub matched_moves: HashMap<ObjectId<Creep>, RoomXY>,
    pub wanted_moves: HashMap<ObjectId<Creep>, RoomXY>,
    pub movement_map: HashMap<RoomXY, ObjectId<Creep>>,
    pub visited_creeps: HashMap<ObjectId<Creep>, bool>,

    pub cached_ops: HashMap<ObjectId<Creep>, Vec<RoomXY>>,

    pub intents: u8,
}

impl RoomMovementCache {
    pub fn new() -> RoomMovementCache {
        RoomMovementCache {
            matched_moves: HashMap::new(),
            wanted_moves: HashMap::new(),
            movement_map: HashMap::new(),
            visited_creeps: HashMap::new(),

            cached_ops: HashMap::new(),

            intents: 0,
        }
    }

    pub fn creep_move(&mut self, creep: &Creep, target_delta: Direction) {
        let current_position = creep.pos();
        let x = current_position.x().u8();
        let y = current_position.y().u8();

        let id = creep.try_id();
        if id.is_none() {
            return;
        }

        let target_position = dir_to_coords(target_delta, x, y);

        let x = target_position.0 as u8;
        let y = target_position.1 as u8;

        if x == 0 || x == 49 || y == 0 || y == 49 {
            return;
        }

        let target_position = unsafe { RoomXY::unchecked_new(x, y) };

        if let std::collections::hash_map::Entry::Vacant(e) = self.wanted_moves.entry(id.unwrap()) {
            e.insert(target_position);
        } else {
            let pos = self.wanted_moves.get_mut(&id.unwrap()).unwrap();
            *pos = target_position;
        }
    }

    pub fn run_room(&mut self, cache: &RoomCache) {
        let mut creeps_with_movement = Vec::new();

        for creep in cache.creeps.creeps.values() {
            let id = creep.try_id();
            if id.is_none() {
                continue;
            }

            if let Some(creep_dest) = self.wanted_moves.get(&id.unwrap()) {
                creeps_with_movement.push((creep, *creep_dest));
            }
        }

        for creep in creeps_with_movement {
            let (creep, target) = creep;

            if self.matched_moves.get(&creep.try_id().unwrap()) == self.wanted_moves.get(&creep.try_id().unwrap()) {
                continue;
            }

            self.movement_map.remove(&target);
            self.matched_moves.remove(&creep.try_id().unwrap());

            if self.depth_first_searh(creep, cache, None) > 0 {
                continue;
            }

            self.assign_creep_to_coord(creep, target);
        }

        for creep in cache.creeps.creeps.values() {
            let matched_move = self.matched_moves.get(&creep.try_id().unwrap());
            if matched_move.is_none() {
                continue;
            }

            if &creep.pos().xy() == matched_move.unwrap() {
                continue;
            }

            let mat = matched_move.unwrap();

            let dir = creep.pos().get_direction_to(Position::new(RoomCoordinate::new(mat.x.u8()).unwrap(), RoomCoordinate::new(mat.y.u8()).unwrap(), creep.room().unwrap().name())).unwrap();
            if creep.move_direction(dir).is_ok() {
                self.intents += 1;
            }
        }
    }

    pub fn depth_first_searh(&mut self, creep: &Creep, cache: &RoomCache, score: Option<i32>) -> i32 {
        let id = creep.try_id();

        self.visited_creeps.entry(id.unwrap()).or_insert(true);

        for roomxy in self.get_possible_moves(creep, cache) {
            let mut score = score.unwrap_or(0);

            if self.wanted_moves.get(&id.unwrap()) == Some(&roomxy) {
                score += 1;
            }

            let occupying_creep = self.movement_map.get(&roomxy);

            if occupying_creep.is_none() {
                if score > 0 {
                    self.assign_creep_to_coord(creep, roomxy);
                }
                return score;
            }

            if !self.visited_creeps.get(occupying_creep.unwrap()).unwrap_or(&false) {
                if self.wanted_moves.get(occupying_creep.unwrap()).unwrap() == &roomxy {
                    score -= 1;
                }

                let result = self.depth_first_searh(creep, cache, Some(score));

                if result > 0 {
                    self.assign_creep_to_coord(creep, roomxy);
                    return result;
                }
            }
        }

        i32::MIN
    }

    pub fn get_possible_moves(&mut self, creep: &Creep, cache: &RoomCache) -> Vec<RoomXY> {
        if let Some(cached) = self.cached_ops.get(&creep.try_id().unwrap()) {
            return cached.clone();
        }

        let mut possible_moves = vec![creep.pos().xy()];

        if creep.tired() {
            return possible_moves;
        }

        if let Some(possible) = self.wanted_moves.get(&creep.try_id().unwrap()) {
            possible_moves.push(*possible);
            return possible_moves;
        }

        let mut positions = vec![];

        let directions = vec![Direction::Top, Direction::TopRight, Direction::Right, Direction::BottomRight, Direction::Bottom, Direction::BottomLeft, Direction::Left, Direction::TopLeft];
        for dir in directions {
            let pos = dir_to_coords(dir, creep.pos().x().u8(), creep.pos().y().u8());
            positions.push(pos);
        }

        let room_terrain = &cache.structures.terrain;

        for pos in positions {

            let roomxy = unsafe { RoomXY::unchecked_new(pos.0, pos.1) };

            let terrain = room_terrain.get_xy(roomxy);
            if terrain == screeps::Terrain::Wall {
                continue;
            }

            possible_moves.push(roomxy);
        }

        possible_moves.shuffle(&mut rand::thread_rng());
        possible_moves
    }

    pub fn assign_creep_to_coord(&mut self, creep: &Creep, coord: RoomXY) {
        let id = creep.try_id();
        if id.is_none() {
            return;
        }

        self.matched_moves.insert(id.unwrap(), coord);
        self.movement_map.insert(coord, id.unwrap());
    }
}