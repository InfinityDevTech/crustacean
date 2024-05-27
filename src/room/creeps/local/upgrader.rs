use screeps::{Creep, HasPosition, MaybeHasId, ResourceType, SharedCreepProperties};

use crate::{memory::ScreepsMemory, room::cache::{hauling::{HaulingPriority, HaulingType}, RoomCache}, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let controller = cache.structures.controller.as_ref().unwrap();

    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    if (creep.store().get_used_capacity(Some(ResourceType::Energy)) as f32) < (creep.store().get_free_capacity(Some(ResourceType::Energy)) as f32 * 0.5) {
        cache.hauling.create_order(creep.try_raw_id().unwrap(), ResourceType::Energy, creep.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap(), HaulingPriority::Energy, HaulingType::Transfer);
    }

    if controller.pos().get_range_to(creep.pos()) > 2 {
        creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), controller.pos(), 2);
    } else {
        let _ = creep.upgrade_controller(controller);
    }
}
