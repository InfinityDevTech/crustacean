use screeps::{CircleStyle, Creep, HasPosition, ResourceType, SharedCreepProperties};

use crate::{
    memory::{CreepMemory, ScreepsMemory},
    room::cache::tick_cache::{CachedRoom, RoomCache},
    traits::creep::CreepExtensions,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_basehauler(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    let room_memory = memory.creeps.get_mut(&creep.name().to_string()).unwrap();
    let room_cache = cache.rooms.get_mut(&room_memory.owning_room).unwrap();

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let _ = creep.say("📋", false);
        find_energy(creep, room_memory, room_cache);
    } else {
        let _ = creep.say("🔋", false);
        deposit_energy(creep, room_memory, room_cache);
    }
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn deposit_energy(creep: &Creep, memory: &mut CreepMemory, room_cache: &mut CachedRoom) {
    // Sort by range.
    let link = room_cache.structures.links.storage.clone();
    let mut extensions = room_cache
        .structures
        .extensions
        .values()
        .collect::<Vec<_>>();
    extensions.sort_by_key(|extension| {
        if extension
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            return u32::MAX;
        }

        extension.pos().get_range_to(creep.pos())
    });
    let mut extensions = extensions.into_iter();

    if let Some(extension) = extensions.next() {
        if extension
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            // Its right next to the storage lmao.
            if let Some(link) = link {
                if link.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                    if creep.pos().is_near_to(link.pos()) {
                        let _ = creep.say("📋 - LINK", false);
                        let _ = creep.transfer(&link, ResourceType::Energy, None);
                    } else {
                        let _ = creep.say("🚚 - LINK", false);
                        creep.better_move_to(memory, room_cache, link.pos(), 1, Default::default());

                        return;
                    }
                }
            }

            if let Some(fastfiller_containers) = &room_cache.structures.containers.fast_filler {
                for container in fastfiller_containers {
                    let container_half = container.store().get_capacity(None) / 2;

                    if container.store().get_used_capacity(None) > container_half {
                        continue;
                    }

                    if creep.pos().is_near_to(container.pos()) {
                        let _ = creep.say("📋 - FASTFILL", false);
                        let _ = creep.transfer(container, ResourceType::Energy, None);
                    } else {
                        let _ = creep.say("🚚 - FASTFILL", false);
                        creep.better_move_to(
                            memory,
                            room_cache,
                            container.pos(),
                            1,
                            Default::default(),
                        );

                        return;
                    }
                }
            }
            return;
        }

        if creep.pos().is_near_to(extension.pos()) {
            let _ = creep.say("📋 - EXT", false);
            let tfer_amount = std::cmp::min(
                creep.store().get_used_capacity(Some(ResourceType::Energy)),
                extension
                    .store()
                    .get_free_capacity(Some(ResourceType::Energy))
                    .try_into()
                    .unwrap(),
            );

            let _ = creep.transfer(extension, ResourceType::Energy, Some(tfer_amount));

            if let Some(next_best) = extensions.next() {
                // Im only checking the store to save ONE. Just ONE. tick of wrong movement
                // But hey, its something :D
                if creep.pos().get_range_to(next_best.pos()) > 1
                    && creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0
                {
                    let _ = creep.say("🚚 - EXT", false);
                    creep.better_move_to(
                        memory,
                        room_cache,
                        next_best.pos(),
                        1,
                        Default::default(),
                    );
                }
            }
        } else {
            let _ = creep.say("🚚 - EXT", false);
            creep.better_move_to(memory, room_cache, extension.pos(), 1, Default::default())
        }
    }

    if let Some(storage) = &room_cache.structures.storage {
        if creep.pos().is_near_to(storage.pos()) && creep.store().get_free_capacity(None) > 0 {
            let _ = creep.say("📋 - STORE", false);
            let _ = creep.withdraw(storage, ResourceType::Energy, None);
        }
    }

    if let Some(storage_link) = &room_cache.structures.links.storage {
        if creep.pos().is_near_to(storage_link.pos()) && storage_link.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
            let _ = creep.say("📋 - LINK", false);
            let _ = creep.transfer(storage_link, ResourceType::Energy, None);
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_energy(creep: &Creep, memory: &mut CreepMemory, room_cache: &mut CachedRoom) {
    if let Some(storage) = &room_cache.structures.storage {
        if storage.store().get_used_capacity(None) > 0 {
            if creep.pos().is_near_to(storage.pos()) {
                let _ = creep.say("📋 - STORE", false);
                let _ = creep.withdraw(
                    storage,
                    ResourceType::Energy,
                    Some(
                        creep
                            .store()
                            .get_free_capacity(Some(ResourceType::Energy))
                            .try_into()
                            .unwrap(),
                    ),
                );
                return;
            } else {
                let _ = creep.say("🚚 - STORE", false);
                creep.better_move_to(memory, room_cache, storage.pos(), 1, Default::default());
                return;
            }
        }
    }

    if let Some(fastfill_containers) = &room_cache.structures.containers.fast_filler {
        let mut most_filled = None;

        for container in fastfill_containers {
            if container.store().get_used_capacity(None) > 0 {
                if most_filled.is_none() {
                    most_filled = Some(container);
                } else if container.store().get_used_capacity(None)
                    > most_filled.unwrap().store().get_used_capacity(None)
                {
                    most_filled = Some(container);
                }
            }
        }

        let most_filled = most_filled.unwrap();

        if creep.pos().is_near_to(most_filled.pos()) {
            let _ = creep.say("📋 - FASTFILL", false);
            let _ = creep.withdraw(
                most_filled,
                ResourceType::Energy,
                Some(
                    creep
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        .try_into()
                        .unwrap(),
                ),
            );
        } else {
            let _ = creep.say("🚚 - FASTFILL", false);
            creep.better_move_to(memory, room_cache, most_filled.pos(), 1, Default::default());
        }
    }
}
