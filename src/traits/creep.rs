use crate::{
    heap, memory::{CreepMemory, ScreepsMemory}, movement::{
        move_target::{MoveOptions, MoveTarget},
        movement_utils::{dir_to_coords, num_to_dir},
    }, room::cache::tick_cache::CachedRoom
};

use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng};
use screeps::{
    game, Direction, HasPosition, MaybeHasId, Position, RoomXY, SharedCreepProperties, Terrain
};

use super::intents_tracking::CreepExtensionsTracking;

pub trait CreepExtensions {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory, cache: &mut CachedRoom);
    fn better_move_to(
        &self,
        creep_memory: &mut ScreepsMemory,
        cache: &mut CachedRoom,
        target: Position,
        range: u16,
        avoid_enemies: MoveOptions,
    );

    fn bsay(&self, message: &str, pub_to_room: bool);

    fn parts_of_type(&self, part: screeps::Part) -> u32;

    fn tired(&self) -> bool;
    fn near_age_death(&self) -> bool;

    fn move_request(&self, target_delta: Direction, room_cache: &mut CachedRoom);
    fn get_possible_moves(&self, room_cache: &mut CachedRoom) -> Vec<RoomXY>;
}


#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CreepExtensions for screeps::Creep {
    // Movement
    fn  better_move_by_path(&self, path: String, memory: &mut CreepMemory, cache: &mut CachedRoom) {
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

        let target_position = dir_to_coords(step_dir, self.pos().x().u8(), self.pos().y().u8());

        let x = target_position.0 as u8;
        let y = target_position.1 as u8;

        if x == 0 || x == 49 || y == 0 || y == 49 {
            let _ = self.ITmove_direction(step_dir);
        } else {
            self.move_request(step_dir, cache);
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
            memory.path = Some(serialized_path.clone());
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
        memory: &mut ScreepsMemory,
        cache: &mut CachedRoom,
        target: Position,
        range: u16,
        move_options: MoveOptions,
    ) {
        let pre_move_cpu = game::cpu::get_used();

        if self.tired() {
            return;
        }

        let creep_memory = memory.creeps.get_mut(&self.name()).unwrap();

        match &creep_memory.path {
            Some(path) => {
                self.better_move_by_path(path.to_string(), creep_memory, cache);
            }
            None => {
                let target = MoveTarget {
                    pos: target,
                    range: range.into(),
                }
                .find_path_to(self.pos(), memory, move_options);

                let creep_memory = memory.creeps.get_mut(&self.name()).unwrap();
                creep_memory.path = Some(target.clone());

                self.better_move_by_path(target, creep_memory, cache);
            }
        }

        cache.stats.global_pathfinding += game::cpu::get_used() - pre_move_cpu;
    }

    fn bsay(&self, message: &str, pub_to_room: bool) {
        let csay = heap().creep_say.lock().unwrap();

        if *csay {
            let _ = self.say(message, pub_to_room);
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

    fn move_request(&self, target_delta: Direction, room_cache: &mut CachedRoom) {
        let current_position = self.pos();
        let x = current_position.x().u8();
        let y = current_position.y().u8();

        //let to = dir_to_coords(target_delta, self.pos().x().u8(), self.pos().y().u8());
        //self.room().unwrap().visual().line(
        //    (self.pos().x().u8() as f32, self.pos().y().u8() as f32),
        //    (to.0 as f32, to.1 as f32),
        //    None,
        //);
        //self.room()
        //    .unwrap()
        //    .visual()
        //    .circle(to.0 as f32, to.1 as f32, None);

        let Some(id) = self.try_id() else { return };

        let target_position = dir_to_coords(target_delta, x, y);
        let target_position = unsafe { RoomXY::unchecked_new(target_position.0, target_position.1) };

        if target_position == self.pos().xy() {
            return;
        }

        if let std::collections::hash_map::Entry::Vacant(e) =
            room_cache.traffic.intended_move.entry(id)
        {
            e.insert(target_position);
        } else {
            let pos = room_cache.traffic.intended_move.get_mut(&id).unwrap();
            *pos = target_position;
        }
    }

    fn get_possible_moves(&self, room_cache: &mut CachedRoom) -> Vec<RoomXY> {
        if room_cache
            .traffic
            .cached_ops
            .contains_key(&self.try_id().unwrap())
        {
            return room_cache.traffic.cached_ops[&self.try_id().unwrap()].clone();
        }

        let mut possible_moves = vec![];

        if self.tired() {
            return possible_moves;
        }

        if room_cache
            .traffic
            .intended_move
            .contains_key(&self.try_id().unwrap())
        {
            possible_moves.insert(
                0,
                *room_cache
                    .traffic
                    .intended_move
                    .get(&self.try_id().unwrap())
                    .unwrap(),
            );
            return possible_moves;
        }

        let mut adjacent_coords = vec![];
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

            adjacent_coords.push(pos);
        }

        let room_terrain = room_cache.structures.terrain.clone();

        let mut seedable = StdRng::seed_from_u64(game::time().into());
        adjacent_coords.shuffle(&mut seedable);

        for coord in adjacent_coords {
            let xy = unsafe { RoomXY::unchecked_new(coord.0, coord.1) };
            let x = xy.x.u8();
            let y = xy.y.u8();

            if room_terrain.get_xy(xy) == Terrain::Wall {
                continue;
            }

            if x == 0 || x == 49 || y == 0 || y == 49 {
                continue;
            }

            possible_moves.push(xy);
        }

        possible_moves.shuffle(&mut seedable);
        possible_moves
    }
}
