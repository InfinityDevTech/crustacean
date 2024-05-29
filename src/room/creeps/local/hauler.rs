use std::vec;

use log::info;
use screeps::{
    game, Creep, HasPosition, HasStore, ObjectId, Resource, ResourceType, Ruin, SharedCreepProperties, Structure, StructureContainer, StructureObject, StructureStorage
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
        execute_order(creep, memory.creeps.get_mut(&creep.name()).unwrap(), order);
    } else {
        let _ = creep.say("📋", false);

        let new_order = if memory.creeps.get(&creep.name()).unwrap().needs_energy.unwrap_or(false) {
            cache.hauling.find_new_order(creep, memory, None, vec![HaulingType::Pickup, HaulingType::Withdraw, HaulingType::Offer])
        } else {
            cache.hauling.find_new_order(creep, memory, None, vec![HaulingType::Transfer])
        };

        if let Some(order) = new_order {
            execute_order(creep, memory.creeps.get_mut(&creep.name()).unwrap(), &order);
        }

        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

        if creep.store().get_free_capacity(None) == 0 {
            creep_memory.needs_energy = None;
        }

        if creep.store().get_used_capacity(None) == 0 {
            creep_memory.needs_energy = Some(true);
        }
    }
}

pub fn execute_order(creep: &Creep, creep_memory: &mut CreepMemory, order: &CreepHaulTask) {
    let pickup_target = order.target_id;
    let position = order.get_target_position();

    if position.is_none() {
        creep_memory.hauling_task = None;
        return;
    }

    if position.unwrap().get_range_to(creep.pos()) > 1 {
        let _ = creep.say("🚚", false);
        creep.better_move_to(creep_memory, position.unwrap(), 1);
        return;
    }
    let _ = creep.say("📦", false);

    let target = game::get_object_by_id_erased(&pickup_target);
    // || target.as_ref().unwrap().unchecked_ref::<StructureStorage>().store().get_free_capacity(Some(order.resource)) == 0
    if target.is_none() || creep.store().get_free_capacity(None) == 0 {
        creep_memory.hauling_task = None;
        return;
    }

    let mut success = false;

    match order.haul_type {
        HaulingType::Pickup => {
            let resource: Option<Resource> = game::get_object_by_id_typed(&ObjectId::from(pickup_target));
            if let Some(resource) = resource {
                let _ = creep.pickup(&resource);
                success = true;
            }
        }
        HaulingType::Withdraw => {
            if let Some(target) = target {
                let amount = std::cmp::min(creep.store().get_free_capacity(Some(ResourceType::Energy)), order.amount as i32);
                let _ = creep.withdraw(target.unchecked_ref::<StructureStorage>(), order.resource, Some(amount.try_into().unwrap()));
            }
        }
        HaulingType::Transfer => {
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
            if let Some(target) = target {
                let amount = std::cmp::min(creep.store().get_free_capacity(Some(order.resource)), order.amount as i32);
                let _ = creep.withdraw(target.unchecked_ref::<StructureStorage>(), order.resource, Some(amount.try_into().unwrap()));
            }
        }
    };

    if success {
        creep_memory.hauling_task = None;
    }
}
