use screeps::{Creep, HasPosition, MaybeHasId, Part, ResourceType, SharedCreepProperties};

use crate::{memory::{CreepMemory, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::{hauling::{HaulingPriority, HaulingType}, CachedRoom, RoomCache}, traits::{creep::CreepExtensions, room::RoomExtensions}, utils::{get_room_sign, scale_haul_priority}};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_upgrader(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if sign_controller(creep, creep_memory, cached_room) || get_energy(creep, creep_memory, cached_room) {
        return;
    }

    let controller = cached_room.structures.controller.as_ref().unwrap();

    if controller.controller.pos().get_range_to(creep.pos()) > 3 {
        creep.better_move_to(creep_memory, cached_room, controller.controller.pos(), 3, MoveOptions::default());
    } else {
        let _ = creep.upgrade_controller(&controller.controller);

        cached_room.stats.energy.spending_upgrading += energy_spent_upgrading(creep);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_energy(creep: &Creep, creep_memory: &mut CreepMemory, cached_room: &mut CachedRoom) -> bool {
    let controller = cached_room.structures.controller.as_ref().unwrap();

    if (creep.store().get_used_capacity(Some(ResourceType::Energy)) as f32) < (creep.store().get_capacity(Some(ResourceType::Energy)) as f32 * 0.75) {
        if let Some(controller_link) = cached_room.structures.links.controller.as_ref() {
            if controller_link.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {

                if creep.pos().is_near_to(controller_link.pos()) {
                    let _ = creep.withdraw(controller_link, ResourceType::Energy, None);

                    return false;
                } else {
                    creep.better_move_to(creep_memory, cached_room, controller_link.pos(), 1, MoveOptions::default());

                    return true;
                }
            }
            return false;
        }
        let container = &controller.container;
        if let Some(container) = container {

            if creep.pos().get_range_to(container.pos()) > 1 {
                creep.better_move_to(creep_memory, cached_room, container.pos(), 1, MoveOptions::default());
                return true;
            } else {
                let _ = creep.withdraw(container, ResourceType::Energy, None);

                // This is dumb as hell, I can harvest and transfer in the same tick.
                // But I cant upgrade and withdraw in the same tick.
                return false;
            }

        } else {
            let priority = creep.store().get_free_capacity(Some(ResourceType::Energy));

            cached_room.hauling.create_order(creep.try_raw_id().unwrap(), None, Some(ResourceType::Energy), Some(creep.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap()), priority as f32, HaulingType::Transfer);
            return false;
        }
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn sign_controller(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut CachedRoom) -> bool {
    let controller = cache.structures.controller.as_ref().unwrap();

    if !creep.room().unwrap().is_my_sign() {
        if creep.pos().is_near_to(controller.controller.pos()) {
            let _ = creep.sign_controller(&controller.controller, &get_room_sign());
        } else {
            creep.better_move_to(creep_memory, cache, controller.controller.pos(), 1, MoveOptions::default());
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