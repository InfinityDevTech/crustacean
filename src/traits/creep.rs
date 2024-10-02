use std::u8;

use crate::{
    compression::compressed_matrix::CompressedMatrix,
    constants::WALKABLE_STRUCTURES,
    heap,
    heap_cache::RoomHeapFlowCache,
    memory::{CreepMemory, ScreepsMemory},
    movement::{
        caching::generate_storage_path,
        move_target::{MoveOptions, MoveTarget},
        movement_utils::{dir_to_coords, num_to_dir},
    },
    room::cache::CachedRoom,
    utils::new_xy,
};

use log::info;
use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng};
use screeps::{
    game, CircleStyle, Direction, HasPosition, MaybeHasId, Position, RoomXY, SharedCreepProperties, Terrain
};

use super::intents_tracking::CreepExtensionsTracking;

pub trait CreepExtensions {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory, cache: &mut CachedRoom);
    fn move_to_storage(&self, cache: &mut CachedRoom) -> bool;
    fn better_move_to(
        &self,
        memory: &mut ScreepsMemory,
        cache: &mut CachedRoom,
        target: Position,
        range: u16,
        avoid_enemies: MoveOptions,
    );

    fn is_stuck(&self, cache: &mut CachedRoom) -> bool;

    fn bsay(&self, message: &str, public: bool);

    fn parts_of_type(&self, part: screeps::Part) -> u32;

    fn tired(&self) -> bool;
    fn near_age_death(&self) -> bool;

    fn set_working_area(&self, cache: &mut CachedRoom, pos: Position, range: u8);
    fn move_request(&self, target_delta: Direction, room_cache: &mut CachedRoom);
    fn get_possible_moves_traffic(&self, room_cache: &mut CachedRoom) -> Vec<RoomXY>;
    fn get_possible_moves(&self, room_cache: &CachedRoom) -> Vec<Direction>;
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
            .entry(cache.room.name())
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

            self.bsay(&format!("MV-STOR {}", dir).to_string(), false);

            return true;
        } else {
            if game::cpu::bucket() > 1000 {
                let steps = generate_storage_path(&self.room().unwrap(), cache);

                room_ff_cache.storage = Some(steps.clone());

                self.move_request(
                    num_to_dir(steps.get_xy(self.pos().x().u8(), self.pos().y().u8())),
                    cache,
                );

                info!("Generated storage path for room {}", cache.room.name());
            }

            return true;
        }
    }

    fn set_working_area(&self, cache: &mut CachedRoom, pos: Position, range: u8) {
        let entry = cache
            .traffic
            .working_areas
            .entry(self.try_id().unwrap())
            .or_insert((pos, range));

        *entry = (pos, range)
    }

    fn better_move_to(
        &self,
        memory: &mut ScreepsMemory,
        cache: &mut CachedRoom,
        target_pos: Position,
        range: u16,
        move_options: MoveOptions,
    ) {
        let pre_move_cpu = game::cpu::get_used();

        if self.tired() {
            self.bsay("😴", false);
            return;
        }

        let not_on_exit = self.pos().x().u8() != 0
            && self.pos().x().u8() != 49
            && self.pos().y().u8() != 0
            && self.pos().y().u8() != 49;

        if !move_options.ignore_cache && !move_options.fixing_stuck_creeps //&& not_on_exit
        {
            if let Some(storage) = &cache.structures.storage {
                if storage.pos() == target_pos && self.move_to_storage(cache) {
                    if self.is_stuck(cache) {
                        self.bsay("ST-STUCK", false);

                        let opts = move_options.clone().fixing_stuck_creeps(true);
                        self.better_move_to(memory, cache, target_pos, range, opts);

                        return;
                    }

                    //self.bsay(&format!("MV-STOR {}",).to_string(), false);
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
            if memory.rooms.contains_key(&self.room().unwrap().name())
                || memory
                    .remote_rooms
                    .contains_key(&self.room().unwrap().name())
            {
                if let Some(cachable_room_positions) = heap_cache.get(&target_pos.room_name()) {
                    // TODO:
                    // They arent saying anything other than HAS, I dont think its caching it...
                    //self.bsay("HAS", false);
                    // If we can cache to that position, then we do the funni.
                    if cachable_room_positions.contains(&target_pos) {
                        //self.bsay("HAS", false);
                        let mut heap_cache = heap().flow_cache.lock().unwrap();

                        let flow_cache = heap_cache
                            .entry(self.room().unwrap().name())
                            .or_insert_with(RoomHeapFlowCache::new);

                        // If there is a cached path to the target, we use it.
                        let path = flow_cache
                            .paths
                            .entry(target_pos)
                            .or_insert_with(CompressedMatrix::new);

                        if self.is_stuck(cache) {
                            path.set_xy(self.pos().x().u8(), self.pos().y().u8(), 0);

                            let possible_moves = self.get_possible_moves(cache);

                            if let Some(pos) = possible_moves.first() {
                                self.move_request(*pos, cache);

                                return;
                            }

                            self.bsay("FIX-STUCK", false);

                            return;
                        }

                        // If the direction is already cached, move there
                        if let Some(dir) = path.get_dir(self.pos().x().u8(), self.pos().y().u8()) {
                            self.bsay(&format!("MV-CHE-{}", dir).to_string(), false);

                            self.move_request(dir, cache);

                            return;
                        } else {
                            // If not, we generate a path to said target, and cache it.
                            // This is a flow fill though, so over time, it will be cached.
                            let target = MoveTarget {
                                pos: target_pos,
                                range: range.into(),
                            }
                            .caching_pathfind(self.pos(), memory);

                            self.bsay(format!("MV-CA-{}", target.incomplete()).as_str(), false);

                            if !target.incomplete() {
                                // From my testing, the pathfinder returns .first() as the creeps position.
                                // Idk why, but hey, it does!
                                if let Some(first) = target.path().first() {
                                    if *first != self.pos() {
                                        let dir = self.pos().get_direction_to(*first);

                                        if let Some(dir) = dir {
                                            self.move_request(dir, cache);

                                            self.room().unwrap().visual().circle(self.pos().x().u8() as f32, self.pos().y().u8() as f32, Some(CircleStyle::default().fill("#ff0000")));

                                            path.set_xy(
                                                self.pos().x().u8(),
                                                self.pos().y().u8(),
                                                dir as u8,
                                            );
                                        }
                                    } else if let Some(first) = target.path().get(1) {
                                        let dir = self.pos().get_direction_to(*first);

                                        if let Some(dir) = dir {
                                            self.move_request(dir, cache);

                                            self.room().unwrap().visual().circle(self.pos().x().u8() as f32, self.pos().y().u8() as f32, Some(CircleStyle::default().fill("#ff0000")));

                                            path.set_xy(
                                                self.pos().x().u8(),
                                                self.pos().y().u8(),
                                                dir as u8,
                                            );
                                        }
                                    }
                                }

                                for (index, step) in target.path().into_iter().enumerate() {
                                    if target.incomplete() && index >= target.path().len() / 2 {
                                        break;
                                    }

                                    // TODO: Make it do it for every step, even across rooms
                                    // Because it currently only works for the room the creep is in.
                                    // P.S. This was fixed by xTwistedx. (The guy with the bad bot.)
                                    if self.room().unwrap().name() != step.room_name() {
                                        self.bsay(format!("BRK-{}", index).as_str(), false);
                                        break;
                                    }

                                    if let Some(next) = target.path().get(index + 1) {
                                        let dir = step.get_direction_to(*next);

                                        if let Some(dir) = dir {
                                            self.room().unwrap().visual().circle(self.pos().x().u8() as f32, self.pos().y().u8() as f32, Some(CircleStyle::default().fill("#00ff00")));
                                            path.set_xy(step.x().u8(), step.y().u8(), dir as u8);
                                        } else {
                                            info!(
                                                "No dir between points {} and {} at index {}!",
                                                step, next, index
                                            );
                                        }
                                    }
                                }

                                return;
                            } else {
                                self.bsay(
                                    &format!("INCMPLT-{}", target.path().len()).to_string(),
                                    false,
                                );
                            }
                        }
                    }
                } else {
                    let mut locked = heap().needs_cachable_position_generation.lock().unwrap();

                    if !locked.contains(&target_pos.room_name()) {
                        locked.push(target_pos.room_name());
                    }
                }
            }
        }

        let creep_memory = memory.creeps.get_mut(&self.name()).unwrap();

        if self.is_stuck(cache) && !move_options.fixing_stuck_creeps {
            self.bsay("CSTUCK", false);

            let possible_moves = self.get_possible_moves(cache);
            creep_memory.path = None;

            if let Some(pos) = possible_moves.first() {
                self.move_request(*pos, cache);
                return;
            }
        }

        match &creep_memory.path {
            Some(path) => {
                self.better_move_by_path(path.to_string(), creep_memory, cache);
            }
            None => {
                let (target, incomplete) = MoveTarget {
                    pos: target_pos,
                    range: range.into(),
                }
                .find_path_to(self.pos(), memory, move_options);

                //self.bsay("PTHFIND", false);

                if incomplete {
                    self.bsay("INCMPLT", false);
                }

                let creep_memory = memory.creeps.get_mut(&self.name()).unwrap();
                creep_memory.path = Some(target.clone());

                self.better_move_by_path(target, creep_memory, cache);
            }
        }

        cache.stats.global_pathfinding += game::cpu::get_used() - pre_move_cpu;
    }

    fn is_stuck(&self, cache: &mut CachedRoom) -> bool {
        if let Some(heap_creep) = heap().creeps.lock().unwrap().get_mut(&self.name()) {
            heap_creep.update_position(self);
            return heap_creep.stuck;
        }

        false
    }

    fn bsay(&self, message: &str, public: bool) {
        heap()
            .per_tick_creep_says
            .lock()
            .unwrap()
            .insert(self.name().to_string(), (public, message.to_string()));
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

    fn get_possible_moves(&self, room_cache: &CachedRoom) -> Vec<Direction> {
        let mut possible_moves = vec![];
        let terrain = room_cache.structures.terrain.clone();

        let s_at_pos = &room_cache.structures.structures_at_pos;
        let cs_at_pos = &room_cache.structures.csites_at_pos;

        for dir in Direction::iter() {
            let pos = dir_to_coords(*dir, self.pos().x().u8(), self.pos().y().u8());

            // >= because if we are at a 1 pos, it will wrap around to 255.
            if pos.0 == 0 || pos.0 >= 49 || pos.1 == 0 || pos.1 >= 49 {
                continue;
            }

            let xy = new_xy(pos.0, pos.1);

            let mut can_run = true;
            for structure in s_at_pos.get(&xy).unwrap_or(&vec![]) {
                if !WALKABLE_STRUCTURES.contains(structure) {
                    can_run = false;
                }
            }

            for csite in cs_at_pos.get(&xy).unwrap_or(&vec![]) {
                if !WALKABLE_STRUCTURES.contains(csite) {
                    can_run = false;
                }
            }

            if terrain.get_xy(xy) == Terrain::Wall || !can_run {
                continue;
            }

            if room_cache.creeps.creeps_at_pos.contains_key(&xy) {
                continue;
            }

            possible_moves.push(*dir);
        }

        let heap_creep = heap().creeps.lock().unwrap();
        let s_creep = heap_creep.get(&self.name());

        if let Some(s_creep) = s_creep {
            let mut new_list = possible_moves.clone();

            for dir in possible_moves.iter() {
                let coord = dir_to_coords(*dir, self.pos().x().u8(), self.pos().y().u8());

                for pos in s_creep.previous_positions.iter() {
                    if (pos.x().u8(), pos.y().u8()) == coord {
                        new_list.retain(|&x| x != *dir);
                    }
                }
            }

            if new_list.is_empty() {
                return possible_moves;
            } else {
                return new_list;
            }
        }

        possible_moves
    }

    fn get_possible_moves_traffic(&self, room_cache: &mut CachedRoom) -> Vec<RoomXY> {
        if room_cache
            .traffic
            .cached_ops
            .contains_key(&self.try_id().unwrap())
        {
            return room_cache.traffic.cached_ops[&self.try_id().unwrap()].clone();
        }

        let mut possible_moves = vec![self.pos().xy()];
        let mut out_of_area = vec![];
        room_cache
            .traffic
            .cached_ops
            .insert(self.try_id().unwrap(), possible_moves.clone());

        if self.tired() {
            return possible_moves;
        }

        if room_cache
            .traffic
            .intended_move
            .contains_key(&self.try_id().unwrap())
        {
            let mut new = vec![room_cache.traffic.intended_move[&self.try_id().unwrap()]];
            new.extend(possible_moves);

            return new;
        }

        let x = self.pos().x().u8();
        let y = self.pos().y().u8();

        let work = room_cache
            .traffic
            .working_areas
            .get(&self.try_id().unwrap());

        for dir in Direction::iter() {
            let pos = dir_to_coords(*dir, x, y);

            if pos.0 == 0 || pos.0 >= 49 || pos.1 == 0 || pos.1 >= 49 {
                continue;
            }

            let xy = new_xy(pos.0, pos.1);

            if room_cache.structures.terrain.get_xy(xy) == Terrain::Wall {
                continue;
            }

            if let Some((working_pos, working_range)) = work {
                if working_pos.xy().get_range_to(xy) > *working_range as u8 {
                    out_of_area.push(xy);

                    continue;
                }
            }

            possible_moves.push(xy);
        }

        let mut rand = StdRng::seed_from_u64(game::time() as u64);
        possible_moves.shuffle(&mut rand);
        out_of_area.shuffle(&mut rand);

        possible_moves.append(&mut out_of_area);
        possible_moves
    }
}
