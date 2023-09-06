use log::info;
use screeps::{HasPosition, Position, game};

use crate::{
    memory::CreepMemory,
    movement::{
        move_target::MoveTarget,
        utils::{dir_to_coords, num_to_dir, visualise_path},
    }, cache::ScreepsCache,
};

pub trait CreepExtensions {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory);
    fn better_move_to(&self, creep_memory: &mut CreepMemory, cache: &mut ScreepsCache, target: Position, range: u16);

    fn better_is_near(&self, object: Position) -> u64;
}

impl CreepExtensions for screeps::Creep {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory) {
        let start_cpu = game::cpu::get_used();
        let creep = self;

        if creep.fatigue() > 0 {
            return;
        }
        let serialized_path = path;
        let serialized_vec = serialized_path
            .split("")
            .filter(|x| x != &"")
            .map(|x| {
                x.parse::<u8>()
                    .expect(&format!("Failed to parse character as u8 {}", x))
            })
            .collect::<Vec<u8>>();
        if serialized_vec.is_empty() {
            memory.p = None;
            return;
        }
        let step_dir = num_to_dir(serialized_vec[0]);

        match creep.move_direction(step_dir) {
            Ok(_) => {
                let serialized_vec = serialized_vec[1..].to_vec();
                let serialized_path = serialized_vec
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("");
                if serialized_vec.is_empty() {
                    memory.p = None;
                    return;
                } else {
                    memory.p = Some(serialized_path);
                }

                let mut points = vec![];
                let mut cursor = (creep.pos().x().u8() as f32, creep.pos().y().u8() as f32);
                for step in serialized_vec {
                    let dir = num_to_dir(step);
                    let (x, y) = dir_to_coords(dir, cursor.0, cursor.1);
                    points.push((x, y));
                    cursor = (x, y);
                }
            }
            Err(e) => {
                memory.p = None;
                info!("Creep move failed, {:?}", e)
            }
        };
        info!("     Move time (Better move by path) {}", game::cpu::get_used() - start_cpu);
    }
    fn better_move_to(&self, creep_memory: &mut CreepMemory, cache: &mut ScreepsCache, target: Position, range: u16) {
        let creep = self;
        match creep_memory.clone().p {
            Some(path) => {
                visualise_path(
                    path.clone().to_string(),
                    creep.clone().room().unwrap().name().to_string(),
                    (creep.pos().x().u8() as f32, creep.pos().y().u8() as f32),
                );
                self.better_move_by_path(path.clone(), creep_memory);
            }
            None => {
                let target = MoveTarget {
                    pos: target,
                    range: range.into(),
                }
                .find_path_to(creep.pos(), cache);
                creep_memory.p = Some(target.clone());
                visualise_path(
                    target.clone().to_string(),
                    creep.clone().room().unwrap().name().to_string(),
                    (creep.pos().x().u8() as f32, creep.pos().y().u8() as f32),
                );
                self.better_move_by_path(target.clone(), creep_memory);
            }
        }
    }

    fn better_is_near(&self, object: Position) -> u64 {
        let x1 = self.pos().x().u8() as f64;
        let y1 = self.pos().y().u8() as f64;
        let x2 = object.x().u8() as f64;
        let y2 = object.y().u8() as f64;
        std::cmp::max((x1 - x2).abs() as u64, (y1 - y2).abs() as u64)
    }
}
