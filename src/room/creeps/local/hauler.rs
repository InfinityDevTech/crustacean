use screeps::{
    game, Creep, ErrorCode, HasPosition, ObjectId, Resource, SharedCreepProperties, StructureStorage
};

use wasm_bindgen::JsCast;

use crate::{
    memory::{CreepHaulTask, CreepMemory, Role, ScreepsMemory},
    room::cache::tick_cache::{hauling::HaulingType, CachedRoom, RoomCache},
    traits::creep::CreepExtensions,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }
    let creep_name = creep.name();

    let creep_memory = memory.creeps.get_mut(&creep_name).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if let Some(order) = &creep_memory.hauling_task.clone() {
        let _ = creep.say("EXEC", false);

        let creep_memory = memory.creeps.get_mut(&creep_name).unwrap();
        if execute_order(
            creep,
            creep_memory,
            cached_room,
            order,
        ) {
            // Invalidate the path and the hauling task
            // Since it was mission success
            if creep_memory.hauling_task.is_some() {
                cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap().heap_cache.hauling.reserved_orders.remove(&creep_memory.hauling_task.as_ref().unwrap().target_id);

                creep_memory.hauling_task = None;
            }
            creep_memory.path = None;

            run_creep(creep, memory, cache);
        }
    } else {
        let new_order = if creep_memory.needs_energy.unwrap_or(false) {
            let _ = creep.say("📋", false);

            cached_room.hauling.find_new_order(
                creep,
                memory,
                None,
                vec![
                    HaulingType::Pickup,
                    HaulingType::Withdraw,
                    HaulingType::Offer,
                ],
                &mut cached_room.heap_cache
            )
        } else {
            let _ = creep.say("🔋", false);

            cached_room
                .hauling
                .find_new_order(creep, memory, None, vec![HaulingType::Transfer], &mut cached_room.heap_cache)
        };

        if let Some(order) = new_order {
            let _ = creep.say("EXEC", false);

            let creep_memory = memory.creeps.get_mut(&creep_name).unwrap();
            if execute_order(
                creep,
                creep_memory,
                cached_room,
                &order,
            ) {
                // Invalidate the path and the hauling task
                // Since it was mission success
                if creep_memory.hauling_task.is_some() {
                    cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap().heap_cache.hauling.reserved_orders.remove(&creep_memory.hauling_task.as_ref().unwrap().target_id);

                    creep_memory.hauling_task = None;
                }
                creep_memory.path = None;

                run_creep(creep, memory, cache);
            }
        }
    }

    if creep.store().get_free_capacity(None) == 0 {
        let mem = memory.creeps.get_mut(&creep_name).unwrap();
        if mem.needs_energy.is_none() { return }
        mem.needs_energy = None;
        mem.path = None;

        if mem.hauling_task.is_some() {
            cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap().heap_cache.hauling.reserved_orders.remove(&mem.hauling_task.as_ref().unwrap().target_id);

            mem.hauling_task = None;
        }
    }

    if creep.store().get_used_capacity(None) == 0 {
        let mem = memory.creeps.get_mut(&creep_name).unwrap();
        if mem.needs_energy.is_some() { return }
        mem.needs_energy = Some(true);
        mem.path = None;

        if mem.hauling_task.is_some() {
            cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap().heap_cache.hauling.reserved_orders.remove(&mem.hauling_task.as_ref().unwrap().target_id);

            mem.hauling_task = None;
        }
    }
}

pub fn execute_order(
    creep: &Creep,
    creep_memory: &mut CreepMemory,
    cache: &mut CachedRoom,
    order: &CreepHaulTask,
) -> bool {
    let pickup_target = order.target_id;
    let target = game::get_object_by_id_erased(&pickup_target);
    let position = order.get_target_position();

    if position.is_none() || target.is_none() {
        creep_memory.hauling_task = None;
        creep_memory.path = None;

        cache.heap_cache.hauling.reserved_orders.remove(&order.target_id);
        let _ = creep.say("INVLD", false);
        return true;
    }

    if !creep.pos().is_near_to(position.unwrap()) {
        creep.better_move_to(creep_memory, cache, position.unwrap(), 1);
        let _ = match order.haul_type {
            HaulingType::Offer => creep.say("MV-OFFR", false),
            HaulingType::Withdraw => creep.say("MV-WTHD", false),
            HaulingType::Pickup => creep.say("MV-PKUP", false),
            HaulingType::Transfer => creep.say("MV-TFER", false),
        };
        return false;
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
        cache.heap_cache.hauling.reserved_orders.remove(&order.target_id);

        return true;
    } else if result.is_err() {
        match result.err().unwrap() {
            ErrorCode::InvalidTarget => {
                let _ = creep.say("INVLD-TGT", false);
                creep_memory.hauling_task = None;
                creep_memory.path = None;

                cache.heap_cache.hauling.reserved_orders.remove(&order.target_id);
            },
            ErrorCode::Full => {
                let _ = creep.say("FULL", false);
                creep_memory.hauling_task = None;
                creep_memory.path = None;

                cache.heap_cache.hauling.reserved_orders.remove(&order.target_id);
            },
            ErrorCode::NoBodypart => {
                let _ = creep.say("NO-BP", false);
                let _ = creep.suicide();
            },
            _ => {}
        }

        return false;
    }

    false
}
