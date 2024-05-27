use screeps::{
    game, Creep, HasPosition, ObjectId, Resource, ResourceType, SharedCreepProperties, Structure, StructureObject, StructureStorage
};

use wasm_bindgen::JsCast;

use crate::{
    memory::{CreepHaulTask, CreepMemory, ScreepsMemory}, room::cache::{hauling::HaulingType, RoomCache}, traits::creep::CreepExtensions
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get(&creep.name()).unwrap();

    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    let order = &creep_memory.hauling_task.clone();
    if let Some(order) = order {
        execute_order(creep, memory.creeps.get_mut(&creep.name()).unwrap(), order);
    } else {
        let _ = creep.say("📋", false);
        let new_order = cache.hauling.find_new_order(creep, memory, None, None);
        if let Some(order) = new_order {
            execute_order(creep, memory.creeps.get_mut(&creep.name()).unwrap(), &order);
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

    let position = position.unwrap();

    if position.get_range_to(creep.pos()) > 1 {
        let _ = creep.say("🚚", false);
        creep.better_move_to(creep_memory, position, 1);
        return;
    }

    let mut success = false;

    let _ = creep.say("📦", false);

    match order.haul_type {
        HaulingType::Pickup => {
            if position.get_range_to(creep.pos()) <= 1 {
                let resource: Option<Resource> = game::get_object_by_id_typed(&ObjectId::from(pickup_target));
                if let Some(resource) = resource {
                    let _ = creep.pickup(&resource);
                    success = true;
                }
            } else {
                creep.better_move_to(creep_memory, position, 1);
            }
        },
        HaulingType::Withdraw => todo!(),
        HaulingType::Transfer => {
            let target = game::get_object_by_id_erased(&pickup_target);
            if let Some(target) = target {
                let _ = creep.transfer(target.unchecked_ref::<StructureStorage>(), ResourceType::Energy, None);
                success = true;
            }
        },
        HaulingType::Offer => {
            success = true;
        }
    }

    if success {
        creep_memory.hauling_task = None;
    }
}