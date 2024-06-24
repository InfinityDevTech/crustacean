use log::info;
use screeps::{
    game, Creep, ErrorCode, HasPosition, ObjectId, Resource, ResourceType, SharedCreepProperties, StructureStorage
};

use wasm_bindgen::JsCast;

use crate::{
    memory::{CreepHaulTask, CreepMemory, Role, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::{self, tick_cache::{hauling::{HaulTaskRequest, HaulingType}, CachedRoom, RoomCache}}, traits::creep::CreepExtensions
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_hauler(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }
    let creep_name = creep.name();

    let creep_memory = memory.creeps.get_mut(&creep_name).unwrap();
    let cached_room = cache.rooms.get_mut(&creep.room().unwrap().name());
    if cached_room.is_none() {
        return;
    }
    let cached_room = cached_room.unwrap();

    if let Some(order) = &creep_memory.hauling_task.clone() {
        let _ = creep.say("EXEC", false);

        if execute_order(
            creep,
            creep_memory,
            cache,
            order,
        ) {
            // Invalidate the path and the hauling task
            // Since it was mission success
            decide_energy_need(creep, creep_memory, cache);    decide_energy_need(creep, creep_memory, cache);

            run_hauler(creep, memory, cache);
            return;
        }
    } else if creep_memory.needs_energy.unwrap_or(false) {
        let _ = creep.say("📋", false);

        cache.rooms.get_mut(&creep_memory.owning_room).unwrap().hauling.wanting_orders.push(HaulTaskRequest::default().creep_name(creep.name()).haul_type(vec![HaulingType::Pickup, HaulingType::Withdraw, HaulingType::Offer]).finish());
    } else {
        let _ = creep.say("🔋", false);

        cache.rooms.get_mut(&creep_memory.owning_room).unwrap().hauling.wanting_orders.push(HaulTaskRequest::default().creep_name(creep.name()).resource_type(ResourceType::Energy).haul_type(vec![HaulingType::Transfer]).finish());
    }

    decide_energy_need(creep, creep_memory, cache);
}

pub fn decide_energy_need(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut RoomCache) {
    if creep_memory.role == Role::Hauler {
        if creep.store().get_free_capacity(None) == 0 {
            if creep_memory.needs_energy.is_none() { return; }
            creep_memory.needs_energy = None;
            creep_memory.path = None;

            if creep_memory.hauling_task.is_some() {
                cache.rooms.get_mut(&creep_memory.owning_room).unwrap().heap_cache.hauling.reserved_orders.remove(&creep_memory.hauling_task.as_ref().unwrap().target_id);
    
                creep_memory.hauling_task = None;
            }
        }

        if creep.store().get_used_capacity(None) == 0 {
            if creep_memory.needs_energy.is_some() { return; }
            creep_memory.needs_energy = Some(true);
            creep_memory.path = None;

            if creep_memory.hauling_task.is_some() {
                cache.rooms.get_mut(&creep_memory.owning_room).unwrap().heap_cache.hauling.reserved_orders.remove(&creep_memory.hauling_task.as_ref().unwrap().target_id);
    
                creep_memory.hauling_task = None;
            }
        }
    }
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn execute_order(
    creep: &Creep,
    creep_memory: &mut CreepMemory,
    cache: &mut RoomCache,
    order: &CreepHaulTask,
) -> bool {
    let pickup_target = order.target_id;
    let target = game::get_object_by_id_erased(&pickup_target);
    let position = order.get_target_position();

    if position.is_none() || target.is_none() {
        creep_memory.hauling_task = None;
        creep_memory.path = None;

        let _ = creep.say("INVLD", false);
        return true;
    }

    if !creep.pos().is_near_to(position.unwrap()) {
        creep.better_move_to(creep_memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), position.unwrap(), 1, MoveOptions::default());
        let _ = match order.haul_type {
            HaulingType::Offer => creep.say("MV-OFFR", false),
            HaulingType::Withdraw => creep.say("MV-WTHD", false),
            HaulingType::Pickup => creep.say("MV-PKUP", false),
            HaulingType::Transfer => creep.say("MV-TFER", false),
        };
        return false;
    }

    let (amount, result): (i32, Result<(), ErrorCode>) = match order.haul_type {
        HaulingType::Pickup => {
            let _ = creep.say("PKUP", false);

            let resource: Option<Resource> = game::get_object_by_id_typed(&ObjectId::from(pickup_target));
            if let Some(resource) = resource {
                let amount = std::cmp::min(
                    creep.store().get_free_capacity(Some(order.resource)),
                    resource.amount().try_into().unwrap(),
                );

                let result = creep.pickup(&resource);

                (amount, result)
            } else {
                (0, Err(ErrorCode::NotFound))
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

                    (amount, result)
                } else {
                    let result = creep.withdraw(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        None,
                    );

                    (0, result)
                }
            } else {
                (0, Err(ErrorCode::NotFound))
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

                    (amount as i32, result)
                } else {
                    let result = creep.transfer(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        None,
                    );

                    (0, result)
                }
            } else {
                (0, Err(ErrorCode::NotFound))
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

                    (amount, result)
                } else {
                    let result = creep.withdraw(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        None,
                    );

                    (0, result)
                }
            } else {
                (0, Err(ErrorCode::NotFound))
            }
        }
    };

    let room_cache = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if result.is_ok() {
        creep_memory.hauling_task = None;
        creep_memory.path = None;

        if let Some(reservation) = room_cache.heap_cache.hauling.reserved_orders.get_mut(&order.target_id) {
            reservation.reserved_amount -= amount;
            reservation.creeps_assigned.retain(|x| x != &creep.name());

            if reservation.reserved_amount <= 0 {
                room_cache.heap_cache.hauling.reserved_orders.remove(&order.target_id);
            }
        }

        return true;
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
            _ => {
                let _ = creep.say(&format!("{:?}", result.err().unwrap()), false);
                creep_memory.path = None;
                creep_memory.hauling_task = None;
            }
        }

        if let Some(reservation) = room_cache.heap_cache.hauling.reserved_orders.get_mut(&order.target_id) {
            reservation.reserved_amount -= amount;
            reservation.creeps_assigned.retain(|x| x != &creep.name());

            if reservation.reserved_amount <= 0 {
                room_cache.heap_cache.hauling.reserved_orders.remove(&order.target_id);
            }
        }

        creep_memory.path = None;
        creep_memory.hauling_task = None;

        return false;
    }

    if let Some(reservation) = room_cache.heap_cache.hauling.reserved_orders.get_mut(&order.target_id) {
        reservation.reserved_amount -= amount;
        reservation.creeps_assigned.retain(|x| x != &creep.name());

        if reservation.reserved_amount <= 0 {
            room_cache.heap_cache.hauling.reserved_orders.remove(&order.target_id);
        }
    }

    creep_memory.path = None;
    creep_memory.hauling_task = None;

    false
}
