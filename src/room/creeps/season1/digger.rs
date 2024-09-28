use std::{cmp, collections::HashMap, hash::Hash};

use screeps::{creep, find, game, pathfinder::{self, MultiRoomCostResult, PathFinder, SearchOptions}, CircleStyle, CostMatrix, CostMatrixGet, CostMatrixSet, Creep, HasId, HasPosition, LocalCostMatrix, RoomName, SharedCreepProperties, StructureObject, StructureType, StructureWall};

use crate::{memory::ScreepsMemory, movement::move_target::MoveOptions, profiling::timing::PATHFIND_CPU, room::cache::RoomCache, traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking}, utils::new_xy};

pub fn run_digger(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if let Some(flag) = game::flags().get("digHere".to_string()) {
        if creep.room().unwrap().name() != flag.pos().room_name() || creep.pos().get_range_to(flag.pos()) > 15 {
            creep.better_move_to(memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), flag.pos(), 10, MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(true));

            return;
        }

        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

        if let Some(repair_target) = creep_memory.repair_target {
            let obj_id = game::get_object_by_id_typed(&repair_target);

            if let Some(obj_id) = obj_id {
                let pos = obj_id.pos();

                if !creep.pos().is_near_to(pos) {
                    creep.better_move_to(memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), obj_id.pos(), 1, MoveOptions::default());
                } else if let StructureObject::StructureWall(obj_id) = StructureObject::from(obj_id) {
                    creep.dismantle(&obj_id);
                } else {
                    creep_memory.repair_target = None;

                    creep.bsay("none", false);
                }

                return;
            } else {
                creep.bsay("No Object.", false);
            }
        }

        let pre_pathfind_cpu = game::cpu::get_used();
            let search = pathfinder::search(creep.pos(), flag.pos(), 0, Some(SearchOptions::new(|room_name| callback(&room_name).1).max_ops(60000)));
        *PATHFIND_CPU.lock().unwrap() += game::cpu::get_used() - pre_pathfind_cpu;
        let vis = creep.room().unwrap().visual();

        if !search.incomplete() {
            let walls = creep.room().unwrap().find(find::STRUCTURES, None);
            let collector = creep.room().unwrap().find(find::SCORE_COLLECTORS, None);
            let mut sc_walls = HashMap::new();

            for step in search.path() {
                vis.circle(step.x().u8() as f32, step.y().u8() as f32, Some(CircleStyle::default().fill("#ff0000")))
            }

            if let Some(collector) = collector.first() {
                for wall in walls {
                    let damageable = wall.as_attackable();
                    if wall.pos().get_range_to(collector.pos()) > 5 || damageable.is_none() {
                        continue;
                    }

                    sc_walls.insert(wall.pos(), wall);
                }
    
                for step in search.path() {
                    if !sc_walls.contains_key(&step.pos()) {
                        let dir = creep.pos().get_direction_to(step.pos());
                        vis.circle(creep.pos().x().u8() as f32, creep.pos().y().u8() as f32, Some(CircleStyle::default().fill("#00ffff")));
                        vis.circle(step.pos().x().u8() as f32, step.pos().y().u8() as f32, Some(CircleStyle::default().fill("#00ffff")));

                        if let Some(dir) = dir {
                            creep.say(&dir.to_string(), false);
                            creep.ITmove_direction(dir);

                            return;
                        } else {
                            vis.circle(step.pos().x().u8() as f32, step.pos().y().u8() as f32, Some(CircleStyle::default().fill("#0000ff")))
                        }
                    } else {
                        let wall = sc_walls.get(&step.pos()).unwrap();

                        creep.dismantle(wall.as_dismantleable().unwrap());

                        creep_memory.repair_target = Some(wall.as_structure().id());
                        vis.circle(step.pos().x().u8() as f32, step.pos().y().u8() as f32, Some(CircleStyle::default().fill("#00ff00")));

                        return;
                    }
                }
            }
        }
    }
}

pub fn callback(room_name: &RoomName) -> (LocalCostMatrix, MultiRoomCostResult) {
    let mut cm = LocalCostMatrix::new();

    let mut terrain = game::map::get_room_terrain(*room_name).unwrap();

    for x in 0..49 {
        for y in 0..49 {
            let xy = new_xy(x, y);

            match terrain.get_xy(xy) {
                screeps::Terrain::Plain => cm.set_xy(xy, 1),
                screeps::Terrain::Wall => cm.set_xy(xy, 255),
                screeps::Terrain::Swamp => cm.set_xy(xy, 5),
            }
        }
    }

    if let Some(room) = game::rooms().get(*room_name) {
        let collector = room.find(find::SCORE_COLLECTORS, None);

        if let Some(collector) = collector.first() {
            let walls = room.find(find::STRUCTURES, None);

            let mut highest_hp = 0;
            let mut sc_walls = Vec::new();

            for wall in walls {
                let damageable = wall.as_attackable();
                if wall.pos().get_range_to(collector.pos()) > 5 || damageable.is_none() {
                    continue;
                }

                let damageable = damageable.unwrap();

                if damageable.hits() > highest_hp {
                    highest_hp = damageable.hits()
                }

                sc_walls.push(wall);
            }

            for wall in sc_walls {
                let xy = wall.pos().xy();
                let attackable = wall.as_attackable().unwrap();

                let percent: f64 = (254.0_f64).min((attackable.hits() as f64 / highest_hp as f64) * 254.0).floor();

                cm.set_xy(xy, percent as u8);
            }
        }
    }

    (cm.clone(), MultiRoomCostResult::CostMatrix(cm.into()))
}