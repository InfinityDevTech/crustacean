use log::info;
use screeps::{HasPosition, MoveToOptions, Position, SharedCreepProperties};

use crate::{
    memory::CreepMemory,
    movement::{
        move_target::MoveTarget,
        utils::{dir_to_coords, num_to_dir},
    },
};

pub trait CreepExtensions {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory);
    fn better_move_to(&self, creep_memory: &mut CreepMemory, target: Position, range: u16);

    fn parts_of_type(&self, part: screeps::Part) -> u32;

    fn tired(&self) -> bool;
    fn near_age_death(&self) -> bool;
}

impl CreepExtensions for screeps::Creep {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory) {
        let creep = self;

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
    }
    fn better_move_to(&self, creep_memory: &mut CreepMemory, target: Position, range: u16) {
        let creep = self;

        if creep.tired() {
            return;
        }

        match creep_memory.clone().p {
            Some(path) => {
                self.better_move_by_path(path.clone(), creep_memory);
            }
            None => {
                let target = MoveTarget {
                    pos: target,
                    range: range.into(),
                }
                .find_path_to(creep.pos());

                creep_memory.p = Some(target.clone());

                self.better_move_by_path(target.clone(), creep_memory);
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
}
