use screeps::{Creep, HasPosition, MaybeHasId, ResourceType, SharedCreepProperties};

use crate::{memory::ScreepsMemory, room::cache::tick_cache::{hauling::{HaulingPriority, HaulingType}, RoomCache}, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let controller = cache.structures.controller.as_ref().unwrap();

    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let container = &controller.container;
        if let Some(container) = container {

            if creep.pos().get_range_to(container.pos()) > 1 {
                creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), cache, container.pos(), 1);
                return;
            } else {
                let _ = creep.withdraw(container, ResourceType::Energy, None);
            }

        } else {
            cache.hauling.create_order(creep.try_raw_id().unwrap(), ResourceType::Energy, creep.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap(), HaulingPriority::Energy, HaulingType::Transfer);
        }
    }


    if controller.controller.pos().get_range_to(creep.pos()) > 2 {
        creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), cache, controller.controller.pos(), 2);
    } else {
        let _ = creep.upgrade_controller(&controller.controller);
    }
}
