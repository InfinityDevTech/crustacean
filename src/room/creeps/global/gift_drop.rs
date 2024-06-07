use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{find, game, Color, Creep, HasPosition, ResourceType, SharedCreepProperties, StructureProperties, StructureType};

use crate::{
    config, memory::ScreepsMemory, room::{cache::tick_cache::{hauling::HaulingType, RoomCache}, creeps::local::hauler}, traits::creep::CreepExtensions
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.store().get_free_capacity(None) > 0 {
        if let Some(task) = &memory.creeps.get(&creep.name()).unwrap().hauling_task.clone() {
            hauler::execute_order(creep, memory.creeps.get_mut(&creep.name()).unwrap(), cache, task);
        }

        cache.hauling.find_new_order(
            creep,
            memory,
            Some(ResourceType::Energy),
            vec![
                HaulingType::Pickup,
                HaulingType::Withdraw,
                HaulingType::Offer,
            ]);
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if let Some(flag) = game::flags().get("giftbasket".to_string()) {
        if creep.room().unwrap().name() == flag.pos().room_name() {
            if creep.pos().get_range_to(flag.pos()) <= 1 {
                let _ = creep.say("😍", true);
                let _ = creep.suicide();
            } else {
                creep.better_move_to(creep_memory, cache, flag.pos(), 1);
            }
        } else {
            let _ = creep.say("🚚", false);
            creep.better_move_to(creep_memory, cache, flag.pos(), 2);
        }
    } else {
        let _ = creep.say("❓", false);
    }
}
