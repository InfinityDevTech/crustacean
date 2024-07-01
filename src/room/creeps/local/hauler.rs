use log::info;
use screeps::{
    game, Creep, ErrorCode, HasId, HasPosition, ObjectId, Resource, ResourceType,
    SharedCreepProperties, StructureStorage,
};

use wasm_bindgen::JsCast;

use crate::{
    heap,
    memory::{CreepHaulTask, CreepMemory, Role, ScreepsMemory},
    movement::move_target::MoveOptions,
    room::cache::tick_cache::{
        hauling::{HaulTaskRequest, HaulingType},
        CachedRoom, RoomCache,
    },
    traits::creep::CreepExtensions,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_hauler(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }
    let creep_name = creep.name();

    let creep_memory = memory.creeps.get_mut(&creep_name).unwrap();

    if let Some(order) = &creep_memory.hauling_task.clone() {
        let _ = creep.say("EXEC", false);

        if execute_order(creep, creep_memory, cache, order) {
            // Invalidate the path and the hauling task
            // Since it was mission success
            decide_energy_need(creep, creep_memory, cache);

            //run_hauler(creep, memory, cache);
            return;
        }
    }

    decide_energy_need(creep, creep_memory, cache);

    if creep_memory.hauling_task.is_none() {
        if creep_memory.needs_energy.unwrap_or(false) {
            let _ = creep.say("📋", false);

            cache
                .rooms
                .get_mut(&creep_memory.owning_room)
                .unwrap()
                .hauling
                .wanting_orders
                .push(
                    HaulTaskRequest::default()
                        .creep_name(creep.name())
                        .haul_type(vec![
                            HaulingType::Pickup,
                            HaulingType::Withdraw,
                            HaulingType::Offer,
                        ])
                        .finish(),
                );
        } else {
            let _ = creep.say("🔋", false);

            cache
                .rooms
                .get_mut(&creep_memory.owning_room)
                .unwrap()
                .hauling
                .wanting_orders
                .push(
                    HaulTaskRequest::default()
                        .creep_name(creep.name())
                        .resource_type(ResourceType::Energy)
                        .haul_type(vec![HaulingType::Transfer])
                        .finish(),
                );
        }
    }
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn decide_energy_need(creep: &Creep, creep_memory: &mut CreepMemory, _cache: &mut RoomCache) {
    if creep_memory.role == Role::Hauler {
        if creep.store().get_free_capacity(None) == 0 {
            if creep_memory.needs_energy.is_none() {
                return;
            }

            creep_memory.needs_energy = None;
            creep_memory.path = None;
            creep_memory.hauling_task = None;
        }

        if creep.store().get_used_capacity(None) == 0 {
            if creep_memory.needs_energy.is_some() {
                return;
            }
            creep_memory.needs_energy = Some(true);
            creep_memory.path = None;
            creep_memory.hauling_task = None;
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

    cache.rooms.get_mut(&creep_memory.owning_room).unwrap().stats.energy.in_haulers += creep.store().get_used_capacity(Some(ResourceType::Energy));

    let current_room_cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    let t = current_room_cache.resources.dropped_energy.clone();
    let mut close_dropped_resources = t.iter().filter(|r| r.pos().is_near_to(creep.pos()));
    if let Some(resource) = close_dropped_resources.next() {
        let free_capacity = creep.store().get_free_capacity(Some(ResourceType::Energy));

        if free_capacity > 0 {
            let _ = creep.pickup(resource);

            // If our free capacity is less than the resource, e.g. we can't pick it all up
            // Drop our hauling task, drop our reservation, and drop the path.
            if free_capacity < resource.amount().try_into().unwrap() {
                creep_memory.path = None;
                creep_memory.hauling_task = None;

                release_reservation(
                    creep,
                    cache.rooms.get_mut(&creep_memory.owning_room).unwrap(),
                    order,
                    resource.amount().try_into().unwrap(),
                );

                cache.creeps_moving_stuff.insert(creep.name(), true);

                return true;
            } else {
                current_room_cache
                    .resources
                    .dropped_energy
                    .retain(|x| x.id() != resource.id());

                let _ = creep.say("PKUP", false);
                let _ = creep.pickup(resource);

                if let Some(haul_task) = &creep_memory.hauling_task.as_ref().unwrap().amount {
                    creep_memory.hauling_task.as_mut().unwrap().amount =
                        Some(haul_task - free_capacity as u32);
                }

                release_reservation(creep, cache.rooms.get_mut(&creep_memory.owning_room).unwrap(), order, free_capacity);

                cache.creeps_moving_stuff.insert(creep.name(), true);

                return true;
            }
        }
    }

    let mut tombstones = current_room_cache.structures.tombstones.values().filter(|t| {
        t.store().get_used_capacity(Some(ResourceType::Energy)) > 0
            && t.pos().is_near_to(creep.pos())
    });
    if let Some(tombstone) = tombstones.next() {
        let free_capacity = creep.store().get_free_capacity(Some(ResourceType::Energy));
        let amount = std::cmp::min(
            free_capacity,
            tombstone
                .store()
                .get_used_capacity(Some(ResourceType::Energy)) as i32,
        );

        if creep.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
            if free_capacity > amount {
                let _ = creep.withdraw(tombstone, ResourceType::Energy, Some(amount as u32));

                cache.creeps_moving_stuff.insert(creep.name(), true);

                if amount == free_capacity {
                    creep_memory.path = None;
                    creep_memory.hauling_task = None;

                    release_reservation(creep, cache.rooms.get_mut(&creep_memory.owning_room).unwrap(), order, amount);

                    return true;
                }
            } else {
                let _ = creep.say("WTHDW", false);

                let _ = creep.withdraw(tombstone, ResourceType::Energy, None);

                if let Some(haul_task) = &creep_memory.hauling_task.as_ref().unwrap().amount {
                    creep_memory.hauling_task.as_mut().unwrap().amount =
                        Some(haul_task - amount as u32);
                }

                current_room_cache.structures.tombstones.remove(&tombstone.id());

                cache.creeps_moving_stuff.insert(creep.name(), true);

                release_reservation(creep, cache.rooms.get_mut(&creep_memory.owning_room).unwrap(), order, amount);
                return true;
            }
        }
    }

    if !creep.pos().is_near_to(position.unwrap()) {
        creep.better_move_to(
            creep_memory,
            cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
            position.unwrap(),
            1,
            MoveOptions::default().path_age(6),
        );

        let _ = match order.haul_type {
            HaulingType::Offer => creep.say("MV-OFFR", false),
            HaulingType::Withdraw => creep.say("MV-WTHD", false),
            HaulingType::Pickup => creep.say("MV-PKUP", false),
            HaulingType::Transfer => creep.say("MV-TFER", false),

            _ => creep.say("MV-UNK", false),
        };
        return false;
    }

    let (amount, result): (i32, Result<(), ErrorCode>) = match order.haul_type {
        HaulingType::Pickup => {
            let _ = creep.say("PKUP", false);

            let resource: Option<Resource> =
                game::get_object_by_id_typed(&ObjectId::from(pickup_target));
            if let Some(resource) = resource {
                let amount = std::cmp::min(
                    creep.store().get_free_capacity(Some(order.resource)),
                    resource.amount().try_into().unwrap(),
                );

                let result = creep.pickup(&resource);

                cache.creeps_moving_stuff.insert(creep.name(), true);

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

                    cache.creeps_moving_stuff.insert(creep.name(), true);

                    (amount, result)
                } else {
                    let result = creep.withdraw(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        None,
                    );

                    cache.creeps_moving_stuff.insert(creep.name(), true);

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

                    if result.is_ok() {
                        cache.rooms.get_mut(&creep_memory.owning_room).unwrap().stats.energy.deposited_energy += amount;
                    }

                    (amount as i32, result)
                } else {
                    let result = creep.transfer(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        None,
                    );

                    if result.is_ok() {
                        cache.rooms.get_mut(&creep_memory.owning_room).unwrap().stats.energy.deposited_energy += creep.store().get_used_capacity(Some(order.resource)) ;
                    }

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

                    let result = if target
                        .unchecked_ref::<StructureStorage>()
                        .store()
                        .get_used_capacity(Some(order.resource))
                        < amount as u32
                    {
                        Err(ErrorCode::InvalidTarget)
                    } else {
                        cache.creeps_moving_stuff.insert(creep.name(), true);

                        creep.withdraw(
                            target.unchecked_ref::<StructureStorage>(),
                            order.resource,
                            Some(amount.try_into().unwrap()),
                        )
                    };

                    (amount, result)
                } else {
                    cache.creeps_moving_stuff.insert(creep.name(), true);

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

        _ => (0, Ok(())),
    };

    let room_cache = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if result.is_ok() {
        release_reservation(creep, room_cache, order, amount);

        creep_memory.hauling_task = None;
        creep_memory.path = None;

        return true;
    } else if result.is_err() {
        match result.err().unwrap() {
            ErrorCode::InvalidTarget => {
                let _ = creep.say("INVLD-TGT", false);
            }
            ErrorCode::Full => {
                let _ = creep.say("FULL", false);
            }
            ErrorCode::NoBodypart => {
                let _ = creep.say("NO-BP", false);
                let _ = creep.suicide();
            }
            _ => {
                let _ = creep.say(&format!("{:?}", result.err().unwrap()), false);
            }
        }

        creep_memory.hauling_task = None;
        creep_memory.path = None;
        release_reservation(creep, room_cache, order, amount);

        return false;
    }

    false
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn release_reservation(
    creep: &Creep,
    _room_cache: &mut CachedRoom,
    order: &CreepHaulTask,
    amount_hauled: i32,
) {
    let mut heap_hauling = heap().hauling.lock().unwrap();
    if let Some(reservation) = heap_hauling.reserved_orders.get_mut(&order.target_id) {
        reservation.reserved_amount -= amount_hauled;
        reservation.creeps_assigned.retain(|x| x != &creep.name());

        info!("{} is releasing reservation, {:?}", creep.name(), order.haul_type);

        if reservation.reserved_amount <= 0 || reservation.creeps_assigned.is_empty() {
            heap_hauling.reserved_orders.remove(&order.target_id);
        }
    }
}
