use screeps::{find, Creep, HasPosition};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory) {
    if let Some(flag) = screeps::game::flags().values().next() {
        if flag.name().to_string() != "move" && flag.name().to_string() != "attack" {
            return;
        }
        if flag.name().to_string() == "move" && !creep.pos().is_near_to(flag.pos()) {
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
                } else if !creep.pos().is_near_to(flag.pos()) {
                creep.better_move_to(creepmem, flag.pos(), 1);
                }
            }
        }
    }
}
