use screeps::{
    game, Creep, ErrorCode, HasId, HasPosition, ObjectId, Resource, ResourceType,
    SharedCreepProperties, StructureStorage,
};

use wasm_bindgen::JsCast;

use crate::{
    heap,
    memory::{CreepHaulTask, Role, ScreepsMemory},
    movement::move_target::MoveOptions,
    room::cache::{
        hauling::{HaulTaskRequest, HaulingType},
        CachedRoom, RoomCache,
    },
    traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking}, utils,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_hauler(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        creep.bsay("😴", false);
        return;
    }
    let creep_name = creep.name();

    let creep_memory = memory.creeps.get_mut(&creep_name).unwrap();

    if let Some(order) = &creep_memory.hauling_task.clone() {
        creep.bsay("EXEC", false);

        if execute_order(creep, memory, cache, order) {
            // Invalidate the path and the hauling task
            // Since it was mission success
            decide_energy_need(creep, memory, cache);

            //run_hauler(creep, memory, cache);
            return;
        }
    }

    decide_energy_need(creep, memory, cache);

    let creep_memory = memory.creeps.get_mut(&creep_name).unwrap();

    if creep_memory.hauling_task.is_none() {
        if creep_memory.needs_energy.unwrap_or(false) {
            creep.bsay("📋", false);
            let room_cache = cache
            .rooms
            .get_mut(&creep_memory.owning_room)
            .unwrap();

            room_cache.idle_haulers += 1;


            room_cache
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
            creep.bsay("🔋", false);
            let room_cache = cache
            .rooms
            .get_mut(&creep_memory.owning_room)
            .unwrap();

            room_cache.idle_haulers += 1;

            let resource = if utils::contains_other_than(&creep.store(), ResourceType::Energy) {
                let mut most_used = ResourceType::Energy;
                let mut most_used_amount = 0;

                for (resource, amount) in utils::store_to_hashmap(&creep.store()) {
                    if resource == ResourceType::Energy {
                        continue;
                    }

                    if amount > most_used_amount {
                        most_used = resource;
                        most_used_amount = amount;
                    }
                }

                most_used
            } else {
                ResourceType::Energy
            };

            room_cache
                .hauling
                .wanting_orders
                .push(
                    HaulTaskRequest::default()
                        .creep_name(creep.name())
                        .resource_type(resource)
                        .haul_type(vec![HaulingType::Transfer])
                        .finish(),
                );
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn decide_energy_need(creep: &Creep, memory: &mut ScreepsMemory, _cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    if creep_memory.role == Role::Hauler {
        let half_capcaity = creep.store().get_capacity(None) as f32 * 0.5;
        let has_other_than_energy = utils::contains_other_than(&creep.store(), ResourceType::Energy);

        if creep.store().get_used_capacity(None) as f32 > half_capcaity || has_other_than_energy {
            if creep_memory.needs_energy.is_none() {
                return;
            }

            creep_memory.needs_energy = None;
            creep_memory.path = None;
            creep_memory.hauling_task = None;
        }

        if creep.store().get_used_capacity(None) as f32 <= half_capcaity && !has_other_than_energy {
            if creep_memory.needs_energy.is_some() {
                return;
            }
            creep_memory.needs_energy = Some(true);
            creep_memory.path = None;
            creep_memory.hauling_task = None;
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn execute_order(
    creep: &Creep,
    memory: &mut ScreepsMemory,
    cache: &mut RoomCache,
    order: &CreepHaulTask,
) -> bool {
    let pickup_target = order.target_id;
    let target = game::get_object_by_id_erased(&pickup_target);
    let position = order.get_target_position();

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if position.is_none() || target.is_none() {
        creep_memory.hauling_task = None;
        creep_memory.path = None;

        creep.bsay("INVLD", false);
        return true;
    }

    cache.rooms.get_mut(&creep_memory.owning_room).unwrap().stats.energy.in_haulers += creep.store().get_used_capacity(Some(ResourceType::Energy));

    let current_room_cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    let t = current_room_cache.resources.dropped_energy.clone();
    let mut close_dropped_resources = t.iter().filter(|r| r.pos().is_near_to(creep.pos()));
    if let Some(resource) = close_dropped_resources.next() {
        let free_capacity = creep.store().get_free_capacity(Some(ResourceType::Energy));

        if free_capacity > 0 {
            let _ = creep.ITpickup(resource);

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

                creep.bsay("PKUP", false);
                let _ = creep.ITpickup(resource);

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

    // TODO: Make this a funciton
    // also, only run it if we have free capacity
    let mut tombstones = current_room_cache.structures.tombstones().values().filter(|t| {
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
                let _ = creep.ITwithdraw(tombstone, ResourceType::Energy, Some(amount as u32));

                cache.creeps_moving_stuff.insert(creep.name(), true);

                if amount == free_capacity {
                    creep_memory.path = None;
                    creep_memory.hauling_task = None;

                    release_reservation(creep, cache.rooms.get_mut(&creep_memory.owning_room).unwrap(), order, amount);

                    return true;
                }
            } else {
                creep.bsay("WTHDW", false);

                let _ = creep.ITwithdraw(tombstone, ResourceType::Energy, None);

                if let Some(haul_task) = &creep_memory.hauling_task.as_ref().unwrap().amount {
                    creep_memory.hauling_task.as_mut().unwrap().amount =
                        Some(haul_task - amount as u32);
                }

                cache.creeps_moving_stuff.insert(creep.name(), true);

                release_reservation(creep, cache.rooms.get_mut(&creep_memory.owning_room).unwrap(), order, amount);
                return true;
            }
        }
    }

    if order.haul_type == HaulingType::Transfer {
        if let Some(ref target) = target {
            let store = target.unchecked_ref::<StructureStorage>().store();

            if store.get_free_capacity(Some(order.resource)) == 0 {
                creep_memory.hauling_task = None;
                creep_memory.path = None;

                release_reservation(creep, cache.rooms.get_mut(&creep_memory.owning_room).unwrap(), order, 0);

                return true;
            }
        }
    }

    if !creep.pos().is_near_to(position.unwrap()) {
        match order.haul_type {
            HaulingType::Offer => creep.bsay("MV-OFFR", false),
            HaulingType::Withdraw => creep.bsay("MV-WTHD", false),
            HaulingType::Pickup => creep.bsay("MV-PKUP", false),
            HaulingType::Transfer => creep.bsay("MV-TFER", false),

            _ => creep.bsay("MV-UNK", false),
        };

        creep.better_move_to(
            memory,
            cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
            position.unwrap(),
            1,
            MoveOptions::default().path_age(6).avoid_enemies(true).avoid_creeps(true).avoid_hostile_rooms(true).avoid_sitters(true),
        );
        return false;
    }

    let (amount, result): (i32, Result<(), ErrorCode>) = match order.haul_type {
        HaulingType::Pickup => {
            creep.bsay("PKUP", false);

            let resource: Option<Resource> =
                game::get_object_by_id_typed(&ObjectId::from(pickup_target));
            if let Some(resource) = resource {
                let amount = std::cmp::min(
                    creep.store().get_free_capacity(Some(order.resource)),
                    resource.amount().try_into().unwrap_or(0),
                );

                let result = creep.ITpickup(&resource);

                cache.creeps_moving_stuff.insert(creep.name(), true);

                (amount, result)
            } else {
                (0, Err(ErrorCode::NotFound))
            }
        }
        HaulingType::Withdraw => {
            creep.bsay("WTHD", false);

            if let Some(target) = target {
                if let Some(amount) = order.amount {
                    let amount = std::cmp::min(
                        creep.store().get_free_capacity(Some(order.resource)),
                        amount as i32,
                    );
                    let result = creep.ITwithdraw(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        Some(amount.try_into().unwrap_or(0)),
                    );

                    cache.creeps_moving_stuff.insert(creep.name(), true);

                    (amount, result)
                } else {
                    let result = creep.ITwithdraw(
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
            creep.bsay("TFER", false);

            if let Some(target) = target {
                if let Some(amount) = order.amount {
                    let amount = std::cmp::min(
                        creep.store().get_used_capacity(Some(order.resource)),
                        amount,
                    );
                    let result = creep.ITtransfer(
                        target.unchecked_ref::<StructureStorage>(),
                        order.resource,
                        Some(amount),
                    );

                    if result.is_ok() {
                        cache.rooms.get_mut(&creep_memory.owning_room).unwrap().stats.energy.deposited_energy += amount;
                    }

                    (amount as i32, result)
                } else {
                    let result = creep.ITtransfer(
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
            creep.bsay("OFFR", false);

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

                        creep.ITwithdraw(
                            target.unchecked_ref::<StructureStorage>(),
                            order.resource,
                            Some(amount.try_into().unwrap()),
                        )
                    };

                    (amount, result)
                } else {
                    cache.creeps_moving_stuff.insert(creep.name(), true);

                    let result = creep.ITwithdraw(
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
                creep.bsay("INVLD-TGT", false);
            }
            ErrorCode::Full => {
                creep.bsay("FULL", false);
            }
            ErrorCode::NoBodypart => {
                creep.bsay("NO-BP", false);
                let _ = creep.ITsuicide();
            }
            _ => {
                creep.bsay(&format!("{:?}", result.err().unwrap()), false);
            }
        }

        creep_memory.hauling_task = None;
        creep_memory.path = None;
        release_reservation(creep, room_cache, order, amount);

        return false;
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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

        if reservation.reserved_amount <= 0 || reservation.creeps_assigned.is_empty() {
            heap_hauling.reserved_orders.remove(&order.target_id);
        }
    }
}
