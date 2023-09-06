use screeps::{find, Creep, HasPosition, StructureObject, StructureType};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory) {
/*    if let Some(flag) = screeps::game::flags()
        .values()
        .find(|f| f.name().to_string() == "move" || f.name().to_string() == "attack")
    {
        if flag.name().to_string() != "move" && flag.name().to_string() != "attack" {
            return;
        }
        if flag.name().to_string() == "move" && !creep.pos().is_near_to(flag.pos()) {
            creep.better_move_to(creepmem, flag.pos(), 1);
        } else if flag.name().to_string() == "attack" {
            let room = creep.room().unwrap();
            let hostile_creeps = room.find(find::HOSTILE_CREEPS, None);
            if !hostile_creeps.is_empty() {
                let hostile = hostile_creeps.first().unwrap();
                if creep.pos().is_near_to(hostile.pos()) {
                    let _ = creep.attack(hostile);
                } else {
                    creep.better_move_to(creepmem, hostile.pos(), 1);
                }
                return
            } else if !creep.pos().is_near_to(flag.pos()) {
                creep.better_move_to(creepmem, flag.pos(), 1);
                return;
            }
            let hostile_structs: Vec<StructureObject> = room
                .find(find::HOSTILE_STRUCTURES, None)
                .into_iter()
                .filter(|c| {
                    c.as_structure().structure_type() != StructureType::Controller
                        || c.as_structure().structure_type() != StructureType::Spawn ||
                        c.as_structure().structure_type() != StructureType::Extension
                })
                .collect();
            let hostile = hostile_structs.first().unwrap();
            if creep.pos().is_near_to(hostile.pos()) {
                let _ = creep.attack(hostile.as_attackable().unwrap());
            } else {
                creep.better_move_to(creepmem, hostile.pos(), 1);
            }
        }
    }
    */
}
