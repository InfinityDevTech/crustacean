use screeps::{find, game, Creep, HasPosition, SharedCreepProperties, StructureProperties, StructureType};

use crate::{
    memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if let Some(flag) = game::flags().get("bulldozeRoom".to_string()) {
        if creep.room().unwrap().name() == flag.pos().room_name() {
            let _ = creep.say("🚜✊", false);
            let enemies = creep.pos().find_closest_by_path(find::HOSTILE_CREEPS, None);
            if let Some(enemy) = enemies {
                if creep.attack(&enemy) == Err(screeps::ErrorCode::NotInRange) {
                    creep.better_move_to(creep_memory, cache, enemy.pos(), 1);
                }
            } else {
                let mut structure = creep.room().unwrap().find(find::HOSTILE_STRUCTURES, None);
                structure.retain(| structure | structure.structure_type() != StructureType::Controller);
                structure.sort_by_key(|structure| structure.pos().get_range_to(creep.pos()));
                let structure = structure.first();
                if let Some(structure) = structure {
                    if creep.pos().is_near_to(structure.pos()) {
                        if let Some(attackabke) = structure.as_attackable() {
                            let _ = creep.attack(attackabke);
                        }
                    } else {
                        creep.better_move_to(creep_memory, cache, structure.pos(), 1);
                    }
                } else {
                    let _ = creep.say("🚚", false);
                    creep.better_move_to(creep_memory, cache, flag.pos(), 3);
                }
            }
        } else {
            let _ = creep.say("🚚", false);
            creep.better_move_to(creep_memory, cache, flag.pos(), 2);
        }
    } else {
        let _ = creep.say("❓", false);
    }
}
