use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{find, game, Color, Creep, HasPosition, ResourceType, SharedCreepProperties, StructureProperties, StructureType};

use crate::{
    config, memory::ScreepsMemory, movement::move_target::MoveOptions, room::{cache::tick_cache::{hauling::{HaulTaskRequest, HaulingType}, CachedRoom, RoomCache}, creeps::local::hauler}, traits::creep::CreepExtensions
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_giftdrop(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if creep.store().get_free_capacity(None) > 0 {

        let room_cache = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

        if let Some(task) = creep_memory.hauling_task.clone() {
            hauler::execute_order(creep, creep_memory, room_cache, &task);

            return;
        }

        room_cache.hauling.wanting_orders.push(HaulTaskRequest::default().creep_name(creep.name()).resource_type(ResourceType::Energy).haul_type(vec![HaulingType::Pickup, HaulingType::Withdraw, HaulingType::Offer]).finish());
        return;
    }

    let cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    if let Some(flag) = game::flags().get("giftbasket".to_string()) {
        if creep.room().unwrap().name() == flag.pos().room_name() {
            if creep.pos().get_range_to(flag.pos()) <= 1 {
                let _ = creep.say("😍", true);
                let _ = creep.suicide();
            } else {
                creep.better_move_to(creep_memory, cache, flag.pos(), 1, MoveOptions::default());
            }
        } else {
            let _ = creep.say("🚚", false);
            creep.better_move_to(creep_memory, cache, flag.pos(), 2, MoveOptions::default());
        }
    } else {
        let _ = creep.say("❓", false);
    }
}
