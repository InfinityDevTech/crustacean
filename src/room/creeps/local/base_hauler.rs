use std::cmp;

use screeps::{Creep, HasPosition, ResourceType, SharedCreepProperties};

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::tick_cache::{CachedRoom, RoomCache},
    traits::creep::CreepExtensions,
};

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_basehauler(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        creep.bsay("😴", false);
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name().to_string()).unwrap();
    let room_cache = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    room_cache.stats.energy.in_base_haulers += creep.store().get_used_capacity(Some(ResourceType::Energy));

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creep.bsay("📋", false);
        find_energy(creep, memory, room_cache);
    } else {
        creep.bsay("🔋", false);
        deposit_energy(creep, memory, room_cache);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn deposit_energy(creep: &Creep, memory: &mut ScreepsMemory, room_cache: &mut CachedRoom) {
    // Sort by range.
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
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

    // If we are next to the storage, withdraw from it.
    // This is done passively.
    if let Some(storage) = &room_cache.structures.storage {
        if creep.pos().is_near_to(storage.pos()) && creep.store().get_free_capacity(None) > 0 {
            creep.bsay("📋 - STORE", false);
            let amount = cmp::min(storage.store().get_used_capacity(Some(ResourceType::Energy)), creep.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap());
            let _ = creep.withdraw(storage, ResourceType::Energy, Some(amount));
        }
    }

    if let Some(extension) = extensions.next() {
        if extension
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            for spawn in room_cache.structures.spawns.values() {
                if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                if creep.pos().is_near_to(spawn.pos()) {
                    creep.bsay("📋 - SPAWN", false);
                    let _ = creep.transfer(spawn, ResourceType::Energy, None);
                } else {
                    creep.bsay("🚚 - SPAWN", false);
                    creep.better_move_to(memory, room_cache, spawn.pos(), 1, Default::default());

                    return;
                }
            }
            }
            // Its right next to the storage lmao.
            if let Some(link) = link {
                let upgrader_count = room_cache.creeps.creeps_of_role.get(&Role::Upgrader).unwrap_or(&Vec::new()).len();
                if link.store().get_free_capacity(Some(ResourceType::Energy)) > 0 && upgrader_count > 0 {
                    if creep.pos().is_near_to(link.pos()) {
                        creep.bsay("📋 - LINK", false);
                        let _ = creep.transfer(&link, ResourceType::Energy, None);
                    } else {
                        creep.bsay("🚚 - LINK", false);
                        creep.better_move_to(memory, room_cache, link.pos(), 1, Default::default());

                        return;
                    }
                }
            }

            if let Some(fastfiller_containers) = &room_cache.structures.containers.fast_filler {
                let lowest = fastfiller_containers.iter().min_by_key(|x| x.store().get_used_capacity(None)).unwrap();

                if lowest.store().get_free_capacity(None) > 0 {
                    if creep.pos().is_near_to(lowest.pos()) {
                        creep.bsay("📋 - FASTFILL", false);
                        let _ = creep.transfer(lowest, ResourceType::Energy, None);
                    } else {
                        creep.bsay("🚚 - FASTFILL", false);
                        creep.better_move_to(
                            memory,
                            room_cache,
                            lowest.pos(),
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
            creep.bsay("📋 - EXT", false);
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
                    creep.bsay("🚚 - EXT", false);
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
            creep.bsay("🚚 - EXT", false);
            creep.better_move_to(memory, room_cache, extension.pos(), 1, Default::default())
        }
    }

    if let Some(storage_link) = &room_cache.structures.links.storage {
        let upgrader_count = room_cache.creeps.creeps_of_role.get(&Role::Upgrader).map_or(0, |x| x.len());
        if creep.pos().is_near_to(storage_link.pos()) && storage_link.store().get_free_capacity(Some(ResourceType::Energy)) > 0 && upgrader_count > 0 && room_cache.structures.storage.as_ref().unwrap().store().get_used_capacity(Some(ResourceType::Energy)) > 10000 {
            creep.bsay("📋 - LINK", false);
            let _ = creep.transfer(storage_link, ResourceType::Energy, None);
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_energy(creep: &Creep, memory: &mut ScreepsMemory, room_cache: &mut CachedRoom) {
    if let Some(storage) = &room_cache.structures.storage {
        if storage.store().get_used_capacity(None) > 0 {
            if creep.pos().is_near_to(storage.pos()) {
                creep.bsay("📋 - STORE", false);
                let amount = cmp::min(storage.store().get_used_capacity(Some(ResourceType::Energy)), creep.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap());
                let _ = creep.withdraw(
                    storage,
                    ResourceType::Energy,
                    Some(amount),
                );
                return;
            } else {
                creep.bsay("🚚 - STORE", false);
                creep.better_move_to(memory, room_cache, storage.pos(), 1, Default::default());
                return;
            }
        }
    }

    if let Some(fastfill_containers) = &room_cache.structures.containers.fast_filler {
        let most_filled = fastfill_containers.iter().max_by_key(|x| x.store().get_used_capacity(None));

        let most_filled = most_filled.unwrap();

        if most_filled.store().get_used_capacity(Some(ResourceType::Energy)) > 0 && creep.pos().is_near_to(most_filled.pos()) {
            creep.bsay("📋 - FASTFILL", false);
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
            creep.bsay("No 🔋?", false);
        }
    }
}
