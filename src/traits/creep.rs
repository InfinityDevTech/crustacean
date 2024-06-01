use crate::{
    memory::CreepMemory,
    movement::{
        move_target::MoveTarget,
        utils::{dir_to_coords, num_to_dir},
    },
    room::cache::tick_cache::RoomCache,
};
use rand::prelude::SliceRandom;
use screeps::{Direction, HasPosition, MaybeHasId, Position, RoomXY};

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
    fn depth_first_searh(&self, room_cache: &mut RoomCache, score: Option<i32>) -> i32;
    fn get_possible_moves(&self, room_cache: &mut RoomCache) -> Vec<RoomXY>;
    fn assign_move_target(&self, room_cache: &mut RoomCache, coord: RoomXY);
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

        self.move_request(step_dir, cache);

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

        let Some(id) = self.try_id() else { return };

        let target_position = dir_to_coords(target_delta, x, y);

        let x = target_position.0 as u8;
        let y = target_position.1 as u8;

        if x == 0 || x == 49 || y == 0 || y == 49 {
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

    fn depth_first_searh(&self, room_cache: &mut RoomCache, score: Option<i32>) -> i32 {
        let id = self.try_id();

        if room_cache.traffic.visited_creeps.clone().is_some() {
            let mut visited = &mut room_cache.traffic.visited_creeps;
            visited.as_mut()
            .unwrap()
            .insert(id.unwrap(), true);
        } else {
            room_cache.traffic.visited_creeps = Some(std::collections::HashMap::new());

            let mut visited = &mut room_cache.traffic.visited_creeps;
            visited.as_mut()
            .unwrap()
            .insert(id.unwrap(), true);
        }

        for roomxy in self.get_possible_moves(room_cache) {
            let mut score = score.unwrap_or(0);

            if room_cache.traffic.move_requests.get(&id.unwrap()) == Some(&roomxy) {
                score += 1;
            }

            let Some(occupying_creep) = room_cache.traffic.movement_map.get(&roomxy) else {
                if score > 0 {
                    self.assign_move_target(room_cache, roomxy);
                }
                return score;
            };

            if !room_cache
                .traffic
                .visited_creeps
                .as_ref()
                .unwrap()
                .get(occupying_creep)
                .unwrap_or(&false)
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

                let result = self.depth_first_searh(room_cache, Some(score));

                if result > 0 {
                    self.assign_move_target(room_cache, roomxy);
                    return result;
                }
            }
        }

        i32::MIN
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

    fn assign_move_target(&self, room_cache: &mut RoomCache, coord: RoomXY) {
        let id = self.try_id();
        if id.is_none() {
            return;
        }

        room_cache.traffic.move_targets.insert(id.unwrap(), coord);
        room_cache.traffic.movement_map.insert(coord, id.unwrap());
    }
}
