use log::info;
use screeps::{
    game, Creep, ErrorCode, HasPosition, ObjectId, Resource, ResourceType, SharedCreepProperties, StructureStorage
};

use wasm_bindgen::JsCast;

use crate::{
    memory::{CreepHaulTask, CreepMemory, ScreepsMemory},
    room::cache::tick_cache::{hauling::HaulingType, RoomCache},
    traits::creep::CreepExtensions,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }
    let creep_name = creep.name();

    let needs_energy = memory
        .creeps
        .get(&creep_name)
        .unwrap()
        .needs_energy
        .unwrap_or(false);

    if let Some(order) = &memory.creeps.get(&creep_name).unwrap().hauling_task.clone() {
        let _ = creep.say("EXEC", false);
        execute_order(
            creep,
            memory.creeps.get_mut(&creep_name).unwrap(),
            cache,
            order,
        );
    } else {
        let new_order = if needs_energy {
            let _ = creep.say("📋", false);

            cache.hauling.find_new_order(
                creep,
                memory,
                None,
                vec![
                    HaulingType::Pickup,
                    HaulingType::Withdraw,
                    HaulingType::Offer,
                ],
            )
        } else {
            let _ = creep.say("🔋", false);

            cache
                .hauling
                .find_new_order(creep, memory, None, vec![HaulingType::Transfer])
        };

        if let Some(order) = new_order {
            let _ = creep.say("EXEC", false);
            execute_order(
                creep,
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cache,
                &order,
            );
        }
    }

    if creep.store().get_free_capacity(None) == 0 {
        let mem = memory.creeps.get_mut(&creep_name).unwrap();
        if mem.needs_energy.is_none() { return }
        mem.needs_energy = None;
        mem.path = None;

        if mem.hauling_task.is_some() {
            mem.hauling_task = None;
        }
    }

    if creep.store().get_used_capacity(None) == 0 {
        let mem = memory.creeps.get_mut(&creep_name).unwrap();
        if mem.needs_energy.is_some() { return }
        mem.needs_energy = Some(true);
        mem.path = None;

        if mem.hauling_task.is_some() {
            mem.hauling_task = None;
        }
    }
}

pub fn execute_order(
    creep: &Creep,
    creep_memory: &mut CreepMemory,
    cache: &mut RoomCache,
    order: &CreepHaulTask,
) {
    let pickup_target = order.target_id;
    let target = game::get_object_by_id_erased(&pickup_target);
    let position = order.get_target_position();

    if position.is_none() || target.is_none() {
        creep_memory.hauling_task = None;
        let _ = creep.say("INVLD", false);
        return;
    }

    if !creep.pos().is_near_to(position.unwrap()) {
        creep.better_move_to(creep_memory, cache, position.unwrap(), 1);
        let _ = match order.haul_type {
            HaulingType::Offer => creep.say("MV-OFFR", false),
            HaulingType::Withdraw => creep.say("MV-WTHD", false),
            HaulingType::Pickup => creep.say("MV-PKUP", false),
            HaulingType::Transfer => creep.say("MV-TFER", false),
        };
        return;
    }

    let result: Result<(), ErrorCode> = match order.haul_type {
        HaulingType::Pickup => {
            let _ = creep.say("PKUP", false);

            let resource: Option<Resource> = game::get_object_by_id_typed(&ObjectId::from(pickup_target));
            if let Some(resource) = resource {
                creep.pickup(&resource)
            } else {
                Err(ErrorCode::NotFound)
            }
        }
        HaulingType::Withdraw => {
            let _ = creep.say("WTHD", false);

            if let Some(target) = target {
                if let Some(amount) = order.amount {
                    let amount = std::cmp::min(
                        creep.store().get_free_capacity(Some(order.resource)),
                        amount as i32,
                    );
                    let result = creep.withdraw(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        Some(amount.try_into().unwrap()),
                    );

                    result
                } else {
                    let result = creep.withdraw(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        None,
                    );

                    result
                }
            } else {
                Err(ErrorCode::NotFound)
            }
        }
        HaulingType::Transfer => {
            let _ = creep.say("TFER", false);

            if let Some(target) = target {
                if let Some(amount) = order.amount {
                    let amount = std::cmp::min(
                        creep.store().get_used_capacity(Some(order.resource)),
                        amount,
                    );
                    let result = creep.transfer(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        Some(amount),
                    );

                    result
                } else {
                    let result = creep.transfer(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        None,
                    );

                    result
                }
            } else {
                Err(ErrorCode::NotFound)
            }
        }
        HaulingType::Offer => {
            let _ = creep.say("OFFR", false);

            if let Some(target) = target {
                if let Some(amount) = order.amount {
                    let amount = std::cmp::min(
                        creep.store().get_free_capacity(Some(order.resource)),
                        amount as i32,
                    );

                    let result = if target.unchecked_ref::<StructureStorage>().store().get_used_capacity(Some(order.resource)) < amount as u32 {
                        Err(ErrorCode::InvalidTarget)
                    } else {
                        creep.withdraw(
                            target.unchecked_ref::<StructureStorage>(),
                            order.resource,
                            Some(amount.try_into().unwrap()),
                        )
                    };

                    result
                } else {
                    let result = creep.withdraw(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        None,
                    );

                    result
                }
            } else {
                Err(ErrorCode::NotFound)
            }
        }
    };

    if result.is_ok() {
        creep_memory.hauling_task = None;
        creep_memory.path = None;
    } else if result.is_err() {
        match result.err().unwrap() {
            ErrorCode::InvalidTarget => {
                let _ = creep.say("INVLD-TGT", false);
                creep_memory.hauling_task = None;
                creep_memory.path = None;
            },
            ErrorCode::Full => {
                let _ = creep.say("FULL", false);
                creep_memory.hauling_task = None;
                creep_memory.path = None;
            },
            ErrorCode::NoBodypart => {
                let _ = creep.say("NO-BP", false);
                let _ = creep.suicide();
            },
            _ => {}
        }
    }
}
