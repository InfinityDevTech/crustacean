use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use screeps::{pathfinder::SearchResults, Direction, Position, RoomXY};

use super::utils::visualise_path;

pub fn path_cache() -> &'static Mutex<PathCache> {
    static HEAP: OnceLock<Mutex<PathCache>> = OnceLock::new();
    HEAP.get_or_init(|| Mutex::new(PathCache::new()))
}

pub struct PathCache {
    pub source_to_dest: HashMap<(Position, Position), Vec<Position>>,

    pub dest_cache: HashMap<Position, Vec<Position>>,
    pub source_cache: HashMap<Position, Vec<Position>>,
}

impl PathCache {
    pub fn new() -> PathCache {
        PathCache {
            source_to_dest: HashMap::new(),
            dest_cache: HashMap::new(),
            source_cache: HashMap::new(),
        }
    }

    pub fn cache_path(&mut self, source: Position, path: Vec<Position>) {
        let dest = path.last();
        if dest.is_none() {
            return;
        }
        let dest = *dest.unwrap();

        self.source_to_dest.insert((source, dest), path.clone());

        if let Some(dest_cache) = self.dest_cache.get_mut(&dest) {
            dest_cache.push(source);
        } else {
            self.dest_cache.insert(dest, vec![source]);
        }

        if let Some(source_cache) = self.source_cache.get_mut(&source) {
            source_cache.push(dest);
        } else {
            self.source_cache.insert(source, vec![dest]);
        }
    }

    pub fn find_closest_entrance_to_dest(
        &mut self,
        current_pos: Position,
        dest: Position,
    ) -> Option<Position> {
        let entrances = self.dest_cache.get(&dest);

        if let Some(entrances) = entrances {
            let mut closest = None;
            let mut closest_distance = 0;

            for entrance in entrances {
                let distance = dest.get_range_to(current_pos);

                if closest.is_none() || distance < closest_distance {
                    closest = Some(*entrance);
                    closest_distance = distance;
                }
            }

            closest
        } else {
            None
        }
    }

    pub fn find_path_were_on(
        &mut self,
        current_pos: Position,
        dest: Position,
    ) -> Option<Vec<Position>> {
        let paths = self.dest_cache.get(&current_pos);

        if let Some(paths) = paths {
            for dest_pos in paths.clone() {
                let steps = self.get_steps_in_path(current_pos, dest_pos);

                if let Some(steps) = steps {
                    for step in steps.clone() {
                        if current_pos == step {
                            let steps = steps.clone();

                            let mut found_pos = false;
                            let mut current_steps = Vec::new();

                            for step in steps {
                                if found_pos {
                                    current_steps.push(step);
                                }

                                if step == dest {
                                    found_pos = true;
                                }
                            }

                            current_steps.reverse();
                            return Some(current_steps);
                        }
                    }
                }
            }

            None
        } else {
            None
        }
    }

    pub fn find_closest_path_to_dest(
        &mut self,
        source: Position,
        dest: Position,
    ) -> (Option<Position>, Option<Vec<Position>>) {
        let paths = self.dest_cache.get(&dest);

        let mut closest = None;
        let mut closest_distance = u32::MAX;

        if let Some(paths) = paths {
            for source_pos in paths.clone() {
                let steps = self.get_steps_in_path(source_pos, dest).unwrap();
                for step in steps.clone() {
                    let range_to_step = source.get_range_to(step);

                    if closest_distance > range_to_step {
                        closest = Some(source_pos);
                        closest_distance = range_to_step;
                    }
                }
            }

            if let Some(closest) = closest {
                return (Some(closest), self.get_steps_in_path(closest, dest));
            }

            (None, None)
        } else {
            (None, None)
        }
    }

    pub fn get_steps_in_path(&mut self, source: Position, dest: Position) -> Option<Vec<Position>> {
        self.source_to_dest.get(&(source, dest)).cloned()
    }

    pub fn visualise_all_paths(&mut self) {
        for (source, dest) in &self.source_to_dest {
            visualise_path(dest.clone(), source.0, "#00ff00");
        }
    }
}
