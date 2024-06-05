use log::info;
use screeps::{
    find, game, look, CircleStyle, Creep, HasId, HasPosition, MaybeHasId, ObjectId, RawObjectId, ResourceType, Room, RoomPosition, RoomXY, SharedCreepProperties, StructureContainer, StructureExtension, StructureObject, StructureProperties, StructureType
};

use wasm_bindgen::JsCast;

use crate::{
    memory::ScreepsMemory,
    room::cache::tick_cache::{
        hauling::{HaulingPriority, HaulingType},
        RoomCache,
    },
    traits::creep::CreepExtensions, utils::scale_haul_priority,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let fastfiller_container = memory
        .creeps
        .get_mut(&creep.name())
        .unwrap()
        .fastfiller_container;

    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    if check_current_position(creep, memory, cache) {
        return
    }

    self_renew(creep, cache);

    if fastfiller_container.is_none() {
        if let Some(container_id) = find_container(creep, memory, cache) {
            memory
                .creeps
                .get_mut(&creep.name())
                .unwrap()
                .fastfiller_container = Some(container_id);
        }
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let _ = creep.say("WTHD", false);
        let container_id = creep_memory.fastfiller_container;
        if container_id.is_none() {
            let priority = scale_haul_priority(
                creep.store().get_capacity(None),
                creep.store().get_used_capacity(None),
                HaulingPriority::Emergency,
                true
            );

            cache.hauling.create_order(
                creep.try_raw_id().unwrap(),
                Some(ResourceType::Energy),
                Some(creep.store().get_free_capacity(Some(ResourceType::Energy)) as u32),
                priority,
                HaulingType::Transfer,
            );
        } else {
            let container = game::get_object_by_id_typed(&container_id.unwrap());

            // Container gets destroyed...
            if container.is_none() {
                creep_memory.fastfiller_container = None;
                return;
            }

            let container = container.unwrap();

            if creep.pos().is_near_to(container.pos()) {
                let _ = creep.withdraw(&container, ResourceType::Energy, None);
            } else {
                creep.better_move_to(creep_memory, cache, container.pos(), 1);
            }

            return;
        }
    }

    let possible_targets = find_possible_targets(creep, cache);
    if possible_targets.is_empty() {
        return;
    }

    let target_id = possible_targets[0];
    let target = game::get_object_by_id_erased(&target_id).unwrap();

    if creep.pos().is_near_to(target.pos()) {
        let _ = creep.transfer(
            target.unchecked_ref::<StructureExtension>(),
            ResourceType::Energy,
            None,
        );
    } else {
        creep.better_move_to(creep_memory, cache, target.pos(), 1);
    }
}

pub fn self_renew(creep: &Creep, cache: &mut RoomCache) {
    let spawn = cache.structures.spawns.values().next().unwrap();

    if creep.ticks_to_live() < Some(100) && creep.pos().is_near_to(spawn.pos()) {
        let _ = spawn.renew_creep(creep);
    }
}

pub fn find_possible_targets(creep: &Creep, cache: &RoomCache) -> Vec<RawObjectId> {
    let find_call = creep.pos().find_in_range(find::STRUCTURES, 1);

    let mut possible_targets = Vec::new();

    for target in find_call {
        match target {
            StructureObject::StructureExtension(extension) => {
                if extension
                    .store()
                    .get_free_capacity(Some(ResourceType::Energy))
                    > 0
                {
                    possible_targets.push(extension.raw_id());
                }
            }

            StructureObject::StructureSpawn(spawn) => {
                if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                    possible_targets.push(spawn.raw_id());
                }
            }
            _ => {}
        }
    }

    possible_targets
}

pub fn check_current_position(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let current_pos = creep.pos().xy();
    let spawn_pos = cache.structures.spawns.values().next().unwrap().pos().xy();

    let position_1 = RoomPosition::new(
        spawn_pos.x.u8() + 1,
        spawn_pos.y.u8() - 1,
        creep.room().unwrap().name(),
    );
    let position_2 = RoomPosition::new(
        spawn_pos.x.u8() - 1,
        spawn_pos.y.u8() - 1,
        creep.room().unwrap().name(),
    );

    if current_pos != unsafe { RoomXY::unchecked_new(position_1.x(), position_1.y()) }
        && current_pos != unsafe { RoomXY::unchecked_new(position_2.x(), position_2.y()) }
    {
        let _ = creep.say("MV-FFPOS", false);
        let pos_1_creep =
            creep
                .room()
                .unwrap()
                .look_for_at_xy(look::CREEPS, position_1.x(), position_1.y());
        let pos_2_creep =
            creep
                .room()
                .unwrap()
                .look_for_at_xy(look::CREEPS, position_2.x(), position_2.y());

        if pos_1_creep.is_empty() {
            creep.better_move_to(
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cache,
                position_1.into(),
                0,
            );
            return true;
        } else if pos_2_creep.is_empty() {
            creep.better_move_to(
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cache,
                position_2.into(),
                0,
            );
            return true;
        }

        return false;
    }

    false
}

pub fn find_container(
    creep: &Creep,
    memory: &mut ScreepsMemory,
    cache: &mut RoomCache,
) -> Option<ObjectId<StructureContainer>> {
    let possible_containers = creep.pos().find_in_range(find::STRUCTURES, 1);

    for container in possible_containers {
        if let StructureObject::StructureContainer(container) = container {
            return Some(container.id());
        }
    }

    None
}
