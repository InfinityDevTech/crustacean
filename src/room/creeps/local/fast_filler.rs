use screeps::{
    game, look, Creep, HasId, HasPosition, MaybeHasId, ObjectId, RawObjectId, ResourceType,
    RoomPosition, RoomXY, SharedCreepProperties, StructureContainer, StructureExtension,
};

use wasm_bindgen::JsCast;

use crate::{
    memory::ScreepsMemory,
    movement::move_target::MoveOptions,
    room::cache::tick_cache::{
        hauling::{HaulingPriority, HaulingType},
        CachedRoom, RoomCache,
    },
    traits::{
        creep::CreepExtensions,
        intents_tracking::{CreepExtensionsTracking, StructureSpawnExtensionsTracking},
    },
    utils::scale_haul_priority,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_fastfiller(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        creep.bsay("😴", false);
        return;
    }

    let cached_room = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    if check_current_position(creep, memory, cached_room) {
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    let fastfiller_container = creep_memory.fastfiller_container;

    self_renew(creep, cached_room);

    if fastfiller_container.is_none() {
        if let Some(container_id) = find_container(creep, cached_room) {
            creep_memory.fastfiller_container = Some(container_id);
        }
    }

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creep.bsay("WTHD", false);
        let container_id = creep_memory.fastfiller_container;
        let link = &cached_room.structures.links.fast_filler;
        if container_id.is_none() && link.is_none() {
            let priority = scale_haul_priority(
                creep.store().get_capacity(None),
                creep.store().get_used_capacity(None),
                HaulingPriority::Emergency,
                true,
            );

            cached_room.hauling.create_order(
                creep.try_raw_id().unwrap(),
                None,
                Some(ResourceType::Energy),
                Some(creep.store().get_free_capacity(Some(ResourceType::Energy)) as u32),
                priority,
                HaulingType::Transfer,
            );
        } else if link.is_some() && link.as_ref().unwrap().store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
            let link = link.as_ref().unwrap();

            if link.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                if creep.pos().is_near_to(link.pos()) {
                    let _ = creep.ITwithdraw(link, ResourceType::Energy, None);
                } else {
                    creep.better_move_to(
                        memory,
                        cached_room,
                        link.pos(),
                        1,
                        MoveOptions::default(),
                    );
                }
            }
        } else {
            if container_id.is_none() {
                creep_memory.fastfiller_container = None;
                return;
            }

            let container = game::get_object_by_id_typed(&container_id.unwrap());

            // Container gets destroyed...
            if container.is_none() {
                creep_memory.fastfiller_container = None;
                return;
            }

            let container = container.unwrap();

            if container.store().get_used_capacity(None) == 0 {
                if let Some(link) = &cached_room.structures.links.fast_filler {
                    if creep.pos().is_near_to(link.pos()) {
                        let _ = creep.ITwithdraw(link, ResourceType::Energy, None);
                    }
                }
            } else if creep.pos().is_near_to(container.pos()) {
                let _ = creep.ITwithdraw(&container, ResourceType::Energy, None);
            } else {
                creep.better_move_to(
                    memory,
                    cached_room,
                    container.pos(),
                    1,
                    MoveOptions::default(),
                );
            }

            return;
        }
    }

    let possible_targets = find_possible_targets(creep, cached_room);
    if possible_targets.is_empty() {
        return;
    }

    let target_id = possible_targets[0];
    let target = game::get_object_by_id_erased(&target_id).unwrap();

    if creep.pos().is_near_to(target.pos()) {
        let _ = creep.ITtransfer(
            target.unchecked_ref::<StructureExtension>(),
            ResourceType::Energy,
            None,
        );
    } else {
        creep.better_move_to(memory, cached_room, target.pos(), 1, MoveOptions::default());
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn self_renew(creep: &Creep, cache: &mut CachedRoom) {
    let spawn = cache.structures.spawns.values().next().unwrap();

    if creep.ticks_to_live() < Some(100)
        && creep.pos().is_near_to(spawn.pos())
        && spawn.spawning().is_none()
    {
        let _ = spawn.ITrenew_creep(creep);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_possible_targets(creep: &Creep, cache: &CachedRoom) -> Vec<RawObjectId> {
    let mut possible_targets = Vec::new();

    for extension in cache.structures.extensions.values() {
        if creep.pos().in_range_to(extension.pos(), 1)
            && extension
                .store()
                .get_free_capacity(Some(ResourceType::Energy))
                > 0
        {
            possible_targets.push(extension.raw_id());
        }
    }

    for spawn in cache.structures.spawns.values() {
        if creep.pos().in_range_to(spawn.pos(), 1)
            && spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0
        {
            possible_targets.push(spawn.raw_id());
        }
    }

    possible_targets
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn check_current_position(
    creep: &Creep,
    memory: &mut ScreepsMemory,
    cache: &mut CachedRoom,
) -> bool {
    let current_pos = creep.pos().xy();
    let spawn_pos = cache.spawn_center.unwrap();

    let position_1 = RoomPosition::new(
        spawn_pos.x.u8() + 1,
        spawn_pos.y.u8(),
        creep.room().unwrap().name(),
    );
    let position_2 = RoomPosition::new(
        spawn_pos.x.u8() - 1,
        spawn_pos.y.u8(),
        creep.room().unwrap().name(),
    );

    if current_pos != unsafe { RoomXY::unchecked_new(position_1.x(), position_1.y()) }
        && current_pos != unsafe { RoomXY::unchecked_new(position_2.x(), position_2.y()) }
    {
        creep.bsay("MV-FFPOS", false);
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
            creep.better_move_to(memory, cache, position_1.into(), 0, MoveOptions::default());
            return true;
        } else if pos_2_creep.is_empty() {
            creep.better_move_to(memory, cache, position_2.into(), 0, MoveOptions::default());
            return true;
        }

        return false;
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_container(
    creep: &Creep,
    cache: &mut CachedRoom,
) -> Option<ObjectId<StructureContainer>> {
    if let Some(fastfiller_containers) = &cache.structures.containers.fast_filler {
        for container in fastfiller_containers {
            if container.pos().get_range_to(creep.pos()) <= 1 {
                return Some(container.id());
            }
        }
    }

    None
}
