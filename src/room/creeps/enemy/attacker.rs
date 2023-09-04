use log::info;
use screeps::{Creep, StructureController, HasPosition, find};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory) {
    info!("Running attackert");
    if let Some(flag) = screeps::game::flags().values().next() {
        info!("Got flag {}", flag.name());
        if flag.name().to_string() != "move" && flag.name().to_string() != "attack" {return}
        info!("Flag is valid");
        if flag.name().to_string() == "move" && !creep.pos().is_near_to(flag.pos()) {
            info!("Moving");
            creep.better_move_to(creepmem, flag.pos(), 1);
    } else if flag.name().to_string() == "attack" {
        let room = creep.room().unwrap();
        let hostile_structs = room.find(find::HOSTILE_STRUCTURES, None);
        if !hostile_structs.is_empty() {
            let hostile = hostile_structs.first().unwrap();
            if creep.pos().is_near_to(hostile.pos()) {
                let _ = creep.attack(hostile.as_attackable().unwrap());
            } else {
                creep.better_move_to(creepmem, hostile.pos(), 1);
            }
        } else {
            let hostile_creeps = room.find(find::HOSTILE_CREEPS, None);
            if !hostile_creeps.is_empty() {
                let hostile = hostile_creeps.first().unwrap();
                if creep.pos().is_near_to(hostile.pos()) {
                    let _ = creep.attack(hostile);
                } else {
                    creep.better_move_to(creepmem, hostile.pos(), 1);
                }
            } else {
                creep.better_move_to(creepmem, flag.pos(), 1);
            }
        }
    }
    }
}
