use screeps::{Creep, HasPosition, MaybeHasId, Part, ResourceType, SharedCreepProperties};

use crate::{memory::{CreepMemory, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::{hauling::{HaulingPriority, HaulingType}, CachedRoom, RoomCache}, traits::{creep::CreepExtensions, room::RoomExtensions}, utils::{get_room_sign, scale_haul_priority}};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if sign_controller(creep, creep_memory, cached_room) {
        return;
    }

    let controller = cached_room.structures.controller.as_ref().unwrap();

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let container = &controller.container;
        if let Some(container) = container {

            if creep.pos().get_range_to(container.pos()) > 1 {
                creep.better_move_to(creep_memory, cached_room, container.pos(), 1, MoveOptions::default());
                return;
            } else {
                let _ = creep.withdraw(container, ResourceType::Energy, None);
            }

        } else {
            let priority = scale_haul_priority(
                creep.store().get_free_capacity(None) as u32,
                creep.store().get_used_capacity(None),
                HaulingPriority::Upgrading,
                true
            );

            cached_room.hauling.create_order(creep.try_raw_id().unwrap(), Some(ResourceType::Energy), Some(creep.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap()), priority, HaulingType::Transfer);
        }
    }


    if controller.controller.pos().get_range_to(creep.pos()) > 2 {
        creep.better_move_to(creep_memory, cached_room, controller.controller.pos(), 2, MoveOptions::default());
    } else {
        let _ = creep.upgrade_controller(&controller.controller);

        cached_room.stats.energy.spending_upgrading += energy_spent_upgrading(creep);
    }
}

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

pub fn energy_spent_upgrading(creep: &Creep) -> u32 {
    let parts = creep.body().iter().filter(|x| x.part() == Part::Work && x.hits() > 0).count() as u32;

    parts * 2
}