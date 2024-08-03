use screeps::{game, HasPosition};

use crate::{memory::ScreepsMemory, movement::move_target::MoveOptions, room::cache::RoomCache};

use super::movement::move_duo;

pub fn filter_dead_creeps(creeps: Vec<String>) -> Vec<String> {
    let mut new_creeps = Vec::new();

    for creep in creeps {
        let gcreep = game::creeps().get(creep.to_string());

        if let Some(gcreep) = gcreep {
            new_creeps.push(creep);
        }
    }

    new_creeps
}

pub fn run_duos(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let duos = memory.formations.duos.clone();

    for (duo_id, duo_memory) in duos {
        let creeps = duo_memory.creeps;
        let mut gcreeps = Vec::new();

        for creep in creeps {
            let gcreep = game::creeps().get(creep.to_string());

            if let Some(gcreep) = gcreep {
                gcreeps.push(gcreep);
            } else {
                memory.formations.duos.get_mut(&duo_id).unwrap().creeps.retain(|x| *x != creep);
            }
        }


        let dest = game::flags().get("duoPoint".to_string()).unwrap().pos();
        let range = 1;
        let move_options = MoveOptions::default();

        move_duo(gcreeps, memory, cache, dest, range, Some(move_options));
    }
}