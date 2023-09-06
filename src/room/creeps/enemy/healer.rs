use log::info;
use screeps::{find, Creep, HasPosition, SharedCreepProperties};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions, cache::ScreepsCache};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) {
    info!("Healer");
    if let Some(flag) = screeps::game::flags()
        .values()
        .find(|f| f.name().to_string() == "move" || f.name().to_string() == "heal")
    {
        if flag.name().to_string() != "move" && flag.name().to_string() != "heal" {
            return;
        }
        info!("Flag {}", flag.name());
        if flag.name().to_string() == "move" && !creep.pos().is_near_to(flag.pos()) {
            creep.better_move_to(creepmem, cache, flag.pos(), 1);
        } else if flag.name().to_string() == "heal" {
            info!(
                "{} {} {}",
                creep.hits() <= creep.hits_max(),
                creep.hits(),
                creep.hits_max()
            );
            if creep.hits() != creep.hits_max() {
                info!("Healing self");
                let _ = creep.heal(creep);
            } else {
                let my_creep = creep.room().unwrap().find(find::MY_CREEPS, None).into_iter().find(|c| c.hits() != c.hits_max());
                if let Some(my_creep) = my_creep {
                    info!("Found creep, moving {}", my_creep.name());
                    if creep.pos().is_near_to(creep.pos()) {
                        let _ = creep.heal(&my_creep);
                    } else {
                        creep.better_move_to(creepmem, cache, creep.pos(), 1);
                    }
                } else if !creep.pos().is_near_to(flag.pos()) {
                    info!("Moving to flag");
                    creep.better_move_to(creepmem, cache, flag.pos(), 1);
                }
            }
        }
    }
}
