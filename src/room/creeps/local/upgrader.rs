use rand::{rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};
use screeps::{game, Creep, HasPosition, MaybeHasId, ResourceType, SharedCreepProperties};

use crate::{config, memory::ScreepsMemory, room::cache::tick_cache::{hauling::{HaulingPriority, HaulingType}, RoomCache}, traits::creep::CreepExtensions, utils::scale_haul_priority};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    if sign_controller(creep, memory, cache) {
        return;
    }

    let controller = cache.structures.controller.as_ref().unwrap();

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let container = &controller.container;
        if let Some(container) = container {

            if creep.pos().get_range_to(container.pos()) > 1 {
                creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), cache, container.pos(), 1);
                return;
            } else {
                let _ = creep.withdraw(container, ResourceType::Energy, None);
            }

        } else {
            let priority = scale_haul_priority(
                creep.store().get_free_capacity(None) as u32,
                creep.store().get_used_capacity(None),
                HaulingPriority::Energy,
                true
            );

            cache.hauling.create_order(creep.try_raw_id().unwrap(), Some(ResourceType::Energy), Some(creep.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap()), priority, HaulingType::Transfer);
        }
    }


    if controller.controller.pos().get_range_to(creep.pos()) > 2 {
        creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), cache, controller.controller.pos(), 2);
    } else {
        let _ = creep.upgrade_controller(&controller.controller);
    }
}

pub fn sign_controller(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let controller = cache.structures.controller.as_ref().unwrap();

    let mut seedable = StdRng::seed_from_u64(game::time().into());

    let signs = config::ROOM_SIGNS;
    let random = &mut seedable.gen_range(0..signs.len());
    let random = signs.get(*random).unwrap();

    if controller.controller.sign().is_none() {
        if creep.pos().is_near_to(controller.controller.pos()) {
            let _ = creep.sign_controller(&controller.controller, random);
        } else {
            creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), cache, controller.controller.pos(), 1);
        }
        return true;
    }

    if !config::ROOM_SIGNS.contains(&controller.controller.sign().unwrap().text().as_str()) {
        if creep.pos().is_near_to(controller.controller.pos()) {
            let _ = creep.sign_controller(&controller.controller, random);
        } else {
            creep.better_move_to(memory.creeps.get_mut(&creep.name()).unwrap(), cache, controller.controller.pos(), 1);
        }

        return true;
    }

    false
}
