use crate::{
    memory::CreepMemory,
    movement::{
        move_target::MoveTarget,
        utils::{dir_to_coords, num_to_dir},
    },
    room::{cache::tick_cache::RoomCache, planning::creep},
};
use log::info;
use rand::prelude::SliceRandom;
use screeps::{Direction, HasPosition, MaybeHasId, Position, RoomXY};

use super::room::RoomExtensions;

pub trait CreepExtensions {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory, cache: &mut RoomCache);
    fn better_move_to(
        &self,
        creep_memory: &mut CreepMemory,
        cache: &mut RoomCache,
        target: Position,
        range: u16,
    );

    fn parts_of_type(&self, part: screeps::Part) -> u32;

    fn tired(&self) -> bool;
    fn near_age_death(&self) -> bool;

    fn move_request(&self, target_delta: Direction, room_cache: &mut RoomCache);
    fn get_possible_moves(&self, room_cache: &mut RoomCache) -> Vec<RoomXY>;
}

impl CreepExtensions for screeps::Creep {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory, cache: &mut RoomCache) {
        let serialized_path = path;
        let serialized_vec = serialized_path
            .split("")
            .filter(|x| x != &"")
            .map(|x| {
                x.parse::<u8>()
                    .unwrap_or_else(|_| panic!("Failed to parse character as u8 {}", x))
            })
            .collect::<Vec<u8>>();
        if serialized_vec.is_empty() {
            memory.path = None;
            return;
        }
        let step_dir = num_to_dir(serialized_vec[0]);

        if self.room().is_some() && self.room().unwrap().my() {
            self.move_request(step_dir, cache);
        } else {
            self.move_direction(step_dir);
        }

        let serialized_vec = serialized_vec[1..].to_vec();
        let serialized_path = serialized_vec
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("");
        if serialized_vec.is_empty() {
            memory.path = None;
            return;
        } else {
            memory.path = Some(serialized_path);
        }

        let mut points = vec![];
        let mut cursor = (self.pos().x().u8() as f32, self.pos().y().u8() as f32);
        for step in serialized_vec {
            let dir = num_to_dir(step);
            let (x, y) = dir_to_coords(dir, cursor.0 as u8, cursor.1 as u8);
            points.push((x, y));
            cursor = (x as f32, y as f32);
        }
    }

    fn better_move_to(
        &self,
        creep_memory: &mut CreepMemory,
        cache: &mut RoomCache,
        target: Position,
        range: u16,
    ) {
        if self.tired() {
            return;
        }

        match &creep_memory.path {
            Some(path) => {
                self.better_move_by_path(path.to_string(), creep_memory, cache);
            }
            None => {
                let target = MoveTarget {
                    pos: target,
                    range: range.into(),
                }
                .find_path_to(self.pos());

                creep_memory.path = Some(target.clone());

                self.better_move_by_path(target, creep_memory, cache);
            }
        }
    }

    fn parts_of_type(&self, part: screeps::Part) -> u32 {
        self.body().iter().filter(|p| p.part() == part).count() as u32
    }

    fn tired(&self) -> bool {
        self.fatigue() > 0
    }

    fn near_age_death(&self) -> bool {
        if let Some(life_time) = self.ticks_to_live() {
            if life_time < 100 {
                return true;
            }
            false
        } else {
            false
        }
    }

    // Part of harabi's movement code.

    fn move_request(&self, target_delta: Direction, room_cache: &mut RoomCache) {
        let current_position = self.pos();
        let x = current_position.x().u8();
        let y = current_position.y().u8();

        let to = dir_to_coords(target_delta, self.pos().x().u8(), self.pos().y().u8());
        self.room().unwrap().visual().line((self.pos().x().u8() as f32, self.pos().y().u8() as f32), (to.0 as f32, to.1 as f32), None);
        self.room().unwrap().visual().circle(to.0 as f32, to.1 as f32, None);

        let Some(id) = self.try_id() else { return };

        let target_position = dir_to_coords(target_delta, x, y);

        let x = target_position.0 as u8;
        let y = target_position.1 as u8;

        if x == 0 || x == 49 || y == 0 || y == 49 {
            let res = self.move_direction(target_delta);
            if res.is_ok() {
                room_cache.traffic.move_intents += 1;
            } else {
                info!("Creep move failed, {:?}", res.err().unwrap());
            }
            return;
        }

        let target_position = unsafe { RoomXY::unchecked_new(x, y) };

        if let std::collections::hash_map::Entry::Vacant(e) =
            room_cache.traffic.move_requests.entry(id)
        {
            e.insert(target_position);
        } else {
            let pos = room_cache.traffic.move_requests.get_mut(&id).unwrap();
            *pos = target_position;
        }
    }

    fn get_possible_moves(&self, room_cache: &mut RoomCache) -> Vec<RoomXY> {
        if let Some(cached) = room_cache.traffic.cached_ops.get(&self.try_id().unwrap()) {
            return cached.to_vec();
        }
    
        let mut possible_moves = vec![self.pos().xy()];

        if self.tired() {
            return possible_moves;
        }
    
        if let Some(possible) = room_cache.traffic.move_requests.get(&self.try_id().unwrap()) {
            possible_moves.push(*possible);
            return possible_moves;
        }
    
        let mut positions = vec![];
    
        let directions = vec![
            Direction::Top,
            Direction::TopRight,
            Direction::Right,
            Direction::BottomRight,
            Direction::Bottom,
            Direction::BottomLeft,
            Direction::Left,
            Direction::TopLeft,
        ];
        for dir in directions {
            let pos = dir_to_coords(dir, self.pos().x().u8(), self.pos().y().u8());
            positions.push(pos);
        }
    
        let room_terrain = &room_cache.structures.terrain;
    
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

}
