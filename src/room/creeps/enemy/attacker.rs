use log::info;
use screeps::{find, Creep, HasPosition};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions, cache::ScreepsCache};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) {
    if let Some(flag) = screeps::game::flags()
        .values()
        .find(|f| f.name().to_string() == "move" || f.name().to_string() == "attack")
    {
        info!("Flag {}", flag.name().to_string());
        if flag.name().to_string() != "move" && flag.name().to_string() != "attack" {
            return;
        }
        if flag.name().to_string() == "move" && !creep.pos().is_near_to(flag.pos()) {
            info!("Moving");
            creep.better_move_to(creepmem, cache, flag.pos(), 1);
        } else if flag.name().to_string() == "attack" {
            info!("Attacking");

            let hostile_creeps = creep.pos().find_closest_by_range(find::HOSTILE_CREEPS);
            if let Some(hostile) = hostile_creeps {
                info!("Found hostiles!");
                if creep.pos().is_near_to(hostile.pos()) {
                    let _ = creep.attack(&hostile);
                } else {
                    creep.better_move_to(creepmem, cache, hostile.pos(), 1);
                }
            } else if !creep.pos().is_near_to(flag.pos()) {
                creep.better_move_to(creepmem, cache, flag.pos(), 1);
            }
            //let hostile_structs: Vec<StructureObject> = room
            //    .find(find::HOSTILE_STRUCTURES, None)
            //    .into_iter()
            //    .filter(|c| {
            //        c.as_structure().structure_type() != StructureType::Controller
            //            || c.as_structure().structure_type() != StructureType::Spawn ||
            //            c.as_structure().structure_type() != StructureType::Extension
            //    })
            //    .collect();
            //let hostile = hostile_structs.first().unwrap();
            //if creep.pos().is_near_to(hostile.pos()) {
            //    let _ = creep.attack(hostile.as_attackable().unwrap());
            //} else {
            //    creep.better_move_to(creepmem, cache, hostile.pos(), 1);
            //}
        }
    }
}
