use screeps::{Creep, HasPosition, MaybeHasId, Part, ResourceType, SharedCreepProperties};

use crate::{memory::{CreepMemory, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::{self, tick_cache::{hauling::{HaulTaskRequest, HaulingPriority, HaulingType}, CachedRoom, RoomCache}}, traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking, room::RoomExtensions}, utils::{get_room_sign, scale_haul_priority}};

use super::{builder::run_builder, hauler::execute_order};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_upgrader(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        creep.bsay("😴", false);
        return;
    }

    if get_energy(creep, memory, cache) || sign_controller(creep, memory, cache) {
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    let controller = cached_room.structures.controller.as_ref().unwrap();

    if controller.controller.pos().get_range_to(creep.pos()) > 3 {
        creep.better_move_to(memory, cached_room, controller.controller.pos(), 3, MoveOptions::default());
    } else {
        let _ = creep.upgrade_controller(&controller.controller);

        cached_room.stats.energy.spending_upgrading += energy_spent_upgrading(creep);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();
    let controller = cached_room.structures.controller.as_ref().unwrap();

    if creep.room().unwrap().name() != creep_memory.owning_room {
        if let Some(task) = creep_memory.hauling_task.clone() {
            execute_order(creep, memory, cache, &task);
        } else {
            let pos = controller.controller.pos();
            creep.better_move_to(memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), pos, 3, MoveOptions::default());
        }
        return true;
    }

    if (creep.store().get_used_capacity(Some(ResourceType::Energy)) as f32) <= (creep.store().get_capacity(Some(ResourceType::Energy)) as f32 * 0.75) {
        if let Some(controller_link) = cached_room.structures.links.controller.as_ref() {
            if controller_link.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {

                if creep.pos().is_near_to(controller_link.pos()) {
                    let _ = creep.ITwithdraw(controller_link, ResourceType::Energy, None);

                    return false;
                } else {
                    creep.better_move_to(memory, cached_room, controller_link.pos(), 1, MoveOptions::default());

                    return true;
                }
            }
            return false;
        }
        let container = &controller.container;
        if let Some(container) = container {
            if creep.pos().get_range_to(container.pos()) > 1 {
                creep.better_move_to(memory, cached_room, container.pos(), 1, MoveOptions::default());
                return true;
            } else {
                let _ = creep.ITwithdraw(container, ResourceType::Energy, None);

                // This is dumb as hell, I can harvest and transfer in the same tick.
                // But I cant upgrade and withdraw in the same tick.
                return false;
            }

        } else if creep.store().get_used_capacity(None) == 0 {
            let priority = creep.store().get_free_capacity(Some(ResourceType::Energy));

            if cached_room.rcl <= 2 {
                if let Some(task) = creep_memory.hauling_task.clone() {
                    execute_order(creep, memory, cache, &task);

                    return true;
                } else {
                    cached_room.hauling.wanting_orders.push(HaulTaskRequest::default().creep_name(creep.name()).resource_type(ResourceType::Energy).haul_type(vec![HaulingType::Offer, HaulingType::Pickup, HaulingType::Withdraw]).clone());

                    return true;
                }
            }
            cached_room.hauling.create_order(creep.try_raw_id().unwrap(), None, Some(ResourceType::Energy), Some(creep.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap()), priority as f32, HaulingType::Transfer);
            return false;
        }
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn sign_controller(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();
    let controller = cache.structures.controller.as_ref().unwrap();

    if !creep.room().unwrap().is_my_sign() {
        if creep.pos().is_near_to(controller.controller.pos()) {
            let _ = creep.ITsign_controller(&controller.controller, &get_room_sign());
        } else {
            creep.better_move_to(memory, cache, controller.controller.pos(), 1, MoveOptions::default());
        }
        return true;
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn energy_spent_upgrading(creep: &Creep) -> u32 {
    let parts = creep.body().iter().filter(|x| x.part() == Part::Work && x.hits() > 0).count() as u32;

    parts * 2
}