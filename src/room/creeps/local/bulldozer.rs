use screeps::{find, game, Creep, HasPosition, SharedCreepProperties};

use crate::{memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if let Some(flag) = game::flags().get("bulldozeRoom".to_string()) {
        if creep.room().unwrap().name() == flag.pos().room_name() {
            let _ = creep.say("🚜", false);
            let enemies = creep.room().unwrap().find(find::HOSTILE_CREEPS, None);
            if !enemies.is_empty() {
                let enemy = &enemies[0];
                if creep.attack(enemy) == Err(screeps::ErrorCode::NotInRange) {
                    creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), cache, enemy.pos(), 1);
                }
            }
        } else {
            let _ = creep.say("🚚", false);
            creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), cache, flag.pos(), 24);
        }
    } else {
        let _ = creep.say("❓", false);
    }
}