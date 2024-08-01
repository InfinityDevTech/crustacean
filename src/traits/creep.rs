use std::u8;

use crate::{
    heap,
    heap_cache::{CompressedDirectionMatrix, RoomHeapFlowCache},
    memory::{CreepMemory, ScreepsMemory},
    movement::{
        caching::{generate_pathing_targets, generate_storage_path},
        move_target::{MoveOptions, MoveTarget},
        movement_utils::{dir_to_coords, num_to_dir},
    },
    room::cache::tick_cache::CachedRoom,
};

use log::info;
use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng};
use screeps::{
    game, pathfinder::SearchOptions, Direction, HasPosition, MaybeHasId, Position, RoomXY,
    SharedCreepProperties, Terrain,
};

use super::intents_tracking::CreepExtensionsTracking;

pub trait CreepExtensions {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory, cache: &mut CachedRoom);
    fn move_to_storage(&self, cache: &mut CachedRoom) -> bool;
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
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory, cache: &mut CachedRoom) {
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

    fn move_to_storage(&self, cache: &mut CachedRoom) -> bool {
        let mut flow_cache = heap().flow_cache.lock().unwrap();

        let room_ff_cache = flow_cache
            .entry(cache.room_name)
            .or_insert_with(RoomHeapFlowCache::new);

        if room_ff_cache.storage.is_some() {
            let dir = num_to_dir(
                room_ff_cache
                    .storage
                    .as_ref()
                    .unwrap()
                    .get_xy(self.pos().x().u8(), self.pos().y().u8()),
            );
            self.move_request(dir, cache);

            return true;
        } else {
            let steps = generate_storage_path(&self.room().unwrap(), cache);

            room_ff_cache.storage = Some(steps.clone());

            self.move_request(
                num_to_dir(steps.get_xy(self.pos().x().u8(), self.pos().y().u8())),
                cache,
            );

            info!("Generated storage path for room {}", cache.room_name);

            return true;
        }

        false
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
        let creep_memory = memory.creeps.get_mut(&self.name()).unwrap();

        if self.tired() {
            return;
        }

        if let Some(storage) = &cache.structures.storage {
            if storage.pos() == target && self.move_to_storage(cache) {
                self.bsay("MV-CACHED", false);
                return;
            }
        }

        let heap_cache = heap().cachable_positions.lock().unwrap();

        // TODO:
        // Say, were pathing to a source, but were not a 6W harvester. Another creep will join.
        // What do you thinks gonna happen:
        //   A. The other creep will find a spot at the source
        //   B. The creeps are gonna fight over a spot at the source
        //   C. The bot will break
        // if you said, "B", you are correct! Dumbass.
        if let Some(cachable_positions) = heap_cache.get(&target.room_name()) {
            self.bsay("HAS", false);
            // If we can cache to that position, then we do the funni.
            if cachable_positions.contains(&target) {
                let mut heap_cache = heap().flow_cache.lock().unwrap();

                let flow_cache = heap_cache
                    .entry(self.room().unwrap().name())
                    .or_insert_with(RoomHeapFlowCache::new);

                // If there is a cached path to the target, we use it.
                let path = flow_cache
                    .paths
                    .entry(target)
                    .or_insert_with(CompressedDirectionMatrix::new);

                // If the direction is already cached, move there
                if let Some(dir) = path.get_dir(self.pos().x().u8(), self.pos().y().u8()) {
                    self.bsay("MV-NCACHE", false);

                    self.move_request(dir, cache);
                    return;
                } else {
                    // If not, we generate a path to said target, and cache it.
                    // This is a flow fill though, so over time, it will be cached.
                    let target = MoveTarget {
                        pos: target,
                        range: range.into(),
                    }
                    .caching_pathfind(self.pos(), memory);

                    self.bsay("MV-CAPTH", false);

                    if !target.incomplete() {
                        let mut previous_position = self.pos();

                        // For len 1 paths, we can just move to the target.
                        if let Some(path_pos) = target.path().first() {
                            if let Some(dir) = previous_position.get_direction_to(*path_pos) {
                                path.set_xy(
                                    previous_position.x().u8(),
                                    previous_position.y().u8(),
                                    dir as u8,
                                );
                            }
                        }

                        for step in target.path() {
                            if let Some(dir) = previous_position.get_direction_to(step) {
                                path.set_xy(
                                    previous_position.x().u8(),
                                    previous_position.y().u8(),
                                    dir as u8,
                                );
                            }

                            previous_position = step;
                        }
                    }

                    if let Some(pos) = path.get_dir(self.pos().x().u8(), self.pos().y().u8()) {
                        self.move_request(pos, cache);
                    }

                    return;
                }
            }
        } else {
            self.bsay("PUSHING", false);
            let mut locked = heap().needs_cachable_position_generation.lock().unwrap();

            if !locked.contains(&target.room_name()) {
                locked.push(target.room_name());
            }
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
                .find_path_to(self.pos(), memory, move_options);

                let creep_memory = memory.creeps.get_mut(&self.name()).unwrap();
                creep_memory.path = Some(target.clone());

                self.better_move_by_path(target, creep_memory, cache);
            }
        }

        cache.stats.global_pathfinding += game::cpu::get_used() - pre_move_cpu;
    }

    fn bsay(&self, message: &str, public: bool) {
        let csay = heap().creep_say.lock().unwrap();

        if *csay {
            let _ = self.say(message, public);
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
        let target_position =
            unsafe { RoomXY::unchecked_new(target_position.0, target_position.1) };

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
