use std::vec;

use screeps::{
    game, Creep, HasPosition, ObjectId, Resource, ResourceType, SharedCreepProperties, StructureStorage
};

use wasm_bindgen::JsCast;

use crate::{
    memory::{CreepHaulTask, CreepMemory, ScreepsMemory},
    room::cache::{hauling::HaulingType, RoomCache},
    traits::creep::CreepExtensions,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    if let Some(order) = &memory.creeps.get(&creep.name()).unwrap().hauling_task.clone() {
        let _ = creep.say("EXEC", false);
        execute_order(creep, memory.creeps.get_mut(&creep.name()).unwrap(), cache, order);
    } else {
        let new_order = if memory.creeps.get(&creep.name()).unwrap().needs_energy.unwrap_or(false) {
            if creep.store().get_free_capacity(None) == 0 {
                memory.creeps.get_mut(&creep.name()).unwrap().needs_energy = None;
            }

            let _ = creep.say("NEED", false);

            cache.hauling.find_new_order(creep, memory, None, vec![HaulingType::Pickup, HaulingType::Withdraw, HaulingType::Offer])
        } else {
            if creep.store().get_used_capacity(None) == 0 {
                memory.creeps.get_mut(&creep.name()).unwrap().needs_energy = Some(true);
            }

            let _ = creep.say("DUMP", false);

            cache.hauling.find_new_order(creep, memory, None, vec![HaulingType::Transfer])
        };

        if let Some(order) = new_order {
            let _ = creep.say("EXEC", false);
            execute_order(creep, memory.creeps.get_mut(&creep.name()).unwrap(), cache, &order);
        }
    }
}

pub fn execute_order(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut RoomCache, order: &CreepHaulTask) {
    let pickup_target = order.target_id;
    let position = order.get_target_position();

    if position.is_none() {
        creep_memory.hauling_task = None;
        return;
    }

    if position.unwrap().get_range_to(creep.pos()) > 1 {
        let _ = creep.say("🚚", false);
        creep.better_move_to(creep_memory, cache, position.unwrap(), 1);
        return;
    }

    let target = game::get_object_by_id_erased(&pickup_target);
    let creep_full = creep.store().get_free_capacity(None) == 0;
    // || target.as_ref().unwrap().unchecked_ref::<StructureStorage>().store().get_free_capacity(Some(order.resource)) == 0
    if target.is_none() {
        creep_memory.hauling_task = None;
        return;
    }

    if creep_full && order.haul_type != HaulingType::Transfer {
        creep_memory.hauling_task = None;
        return;
    }

    let mut success = false;

    match order.haul_type {
        HaulingType::Pickup => {
            if creep_full { return; }
            let _ = creep.say("PKUP", false);
            let resource: Option<Resource> = game::get_object_by_id_typed(&ObjectId::from(pickup_target));
            if let Some(resource) = resource {
                let _ = creep.pickup(&resource);
                success = true;
            }
        }
        HaulingType::Withdraw => {
            if creep_full { return; }
            let _ = creep.say("WTHD", false);
            if let Some(target) = target {
                let amount = std::cmp::min(creep.store().get_free_capacity(Some(ResourceType::Energy)), order.amount as i32);
                let _ = creep.withdraw(target.unchecked_ref::<StructureStorage>(), order.resource, Some(amount.try_into().unwrap()));
                success = true;
            }
        }
        HaulingType::Transfer => {
            let _ = creep.say("TFER", false);
            if let Some(target) = target {
                let _ = creep.transfer(
                    target.unchecked_ref::<StructureStorage>(),
                    ResourceType::Energy,
                    None,
                );
                success = true;
            }
        }
        HaulingType::Offer => {
            let _ = creep.say("OFFR", false);
            if let Some(target) = target {
                let amount = std::cmp::min(creep.store().get_free_capacity(Some(order.resource)), order.amount as i32);
                let _ = creep.withdraw(target.unchecked_ref::<StructureStorage>(), order.resource, Some(amount.try_into().unwrap()));
                success = true;
            }
        }
    };

    if success {
        creep_memory.hauling_task = None;
    }
}
