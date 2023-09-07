use log::info;
use screeps::{game, HasPosition, Position};

use crate::{
    cache::ScreepsCache,
    memory::CreepMemory,
    movement::{
        move_target::MoveTarget,
        utils::{dir_to_coords, num_to_dir, visualise_path},
    },
};

pub trait CreepExtensions {
    // Movement
    fn better_move_by_path(&self, path: String, memory: &mut CreepMemory);
    fn better_move_to(
        &self,
        creep_memory: &mut CreepMemory,
        cache: &mut ScreepsCache,
        target: Position,
        range: u16,
    );

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
        // Turn to u8's
        let serialized_path = path;
        let serialized_vec = serialized_path
            .split("")
            .filter(|x| x != &"")
            .map(|x| {
                x.parse::<u8>()
                    .expect(&format!("Failed to parse character as u8 {}", x))
            })
            .collect::<Vec<u8>>();
        // Empty
        if serialized_vec.is_empty() {
            memory.p = None;
            return;
        }
        // Get direction
        let step_dir = num_to_dir(serialized_vec[0]);

        match creep.move_direction(step_dir) {
            Ok(_) => {
                let serialized_path = serialized_vec[1..]
                    .to_vec()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("");
                if serialized_vec.is_empty() {
                    memory.p = None;
                } else {
                    memory.p = Some(serialized_path);
                }
            }
            Err(e) => {
                memory.p = None;
                info!("Creep move failed, {:?}", e)
            }
        };
        info!(
            "     Move time (Better move by path) {}",
            game::cpu::get_used() - start_cpu
        );
    }
    fn better_move_to(
        &self,
        creep_memory: &mut CreepMemory,
        cache: &mut ScreepsCache,
        target: Position,
        range: u16,
    ) {
        let starting_cpu = game::cpu::get_used();
        let creep = self;
        match creep_memory.clone().p {
            Some(path) => {
                info!("     Decoding!");
                //visualise_path(
                //    path.clone().to_string(),
                //    creep.clone().room().unwrap().name().to_string(),
                //    (creep.pos().x().u8() as f32, creep.pos().y().u8() as f32),
                //);
                info!("     Draw time {}", game::cpu::get_used() - starting_cpu);
                self.better_move_by_path(path.clone(), creep_memory);
                info!(
                    "     Better move by path time: {}",
                    game::cpu::get_used() - starting_cpu
                );
            }
            None => {
                info!("     Recalculating path");
                let target = MoveTarget {
                    pos: target,
                    range: range.into(),
                }
                .find_path_to(creep.pos(), cache);
                info!("Find time: {}", game::cpu::get_used() - starting_cpu);
                creep_memory.p = Some(target.clone());
                //visualise_path(
                //    target.clone().to_string(),
                //    creep.clone().room().unwrap().name().to_string(),
                //    (creep.pos().x().u8() as f32, creep.pos().y().u8() as f32),
                //);
                info!("     Draw time {}", game::cpu::get_used() - starting_cpu);
                self.better_move_by_path(target.clone(), creep_memory);
                info!(
                    "     Better move by path time: {}",
                    game::cpu::get_used() - starting_cpu
                );
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
