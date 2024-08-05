use screeps::{Creep, HasPosition, Position, ResourceType, SharedCreepProperties};

use crate::{
    memory::{Role, ScreepsMemory},
    movement::move_target::MoveOptions,
    room::cache::{CachedRoom, RoomCache},
    traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking},
    utils,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_storagesitter(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let room_cache = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if check_pos(creep, memory, room_cache) {
        return;
    }

    let storage = if let Some(storage) = &room_cache.structures.storage {
        storage
    } else {
        return;
    };

    // If we have less than 50 TTL, just dump everything into storage and die.
    if creep.ticks_to_live() <= Some(50) {
        if creep.pos().get_range_to(storage.pos()) > 1 {
            creep.better_move_to(memory, room_cache, storage.pos(), 1, MoveOptions::default());
        } else if creep.store().get_used_capacity(None) > 0 {
            let contains = utils::store_to_hashmap(&creep.store());

            for (resource, amount) in contains.iter() {
                let _ = creep.ITtransfer(storage, *resource, Some(*amount));
            }
        }

        return;
    }

    // Storage link stuff, if we have upgraders, dump energy into storage link.
    // If we don't have upgraders, dump energy from storage link into storage.
    if let Some(storage_link) = &room_cache.structures.links().storage {
        let upgrader_count = room_cache
            .creeps
            .creeps_of_role
            .get(&Role::Upgrader)
            .unwrap_or(&Vec::new())
            .len();
        if (upgrader_count >= 1 && room_cache.structures.links().controller.is_some()) && storage.store().get_used_capacity(Some(ResourceType::Energy)) > 20000 {
            if storage_link
                .store()
                .get_free_capacity(Some(ResourceType::Energy))
                > 0
            {
                if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
                    let _ = creep.ITwithdraw(storage, ResourceType::Energy, None);

                    return;
                } else {
                    let _ = creep.ITtransfer(storage_link, ResourceType::Energy, None);

                    return;
                }
            }
        } else if storage_link
            .store()
            .get_used_capacity(Some(ResourceType::Energy)) > 0 && creep.store().get_used_capacity(Some(ResourceType::Energy)) < creep.store().get_capacity(Some(ResourceType::Energy)) {
            let _ = creep.ITwithdraw(storage_link, ResourceType::Energy, None);

            return;
        }
    }
    // end storage link stuff.

    // Nuker stuffs, if storage has 110,000 energy, dump it into nuker (if we can)
    if let Some(nuker) = &room_cache.structures.nuker {
        if nuker
            .store()
            .get_free_capacity(Some(screeps::ResourceType::Energy))
            > 0
            && storage
                .store()
                .get_used_capacity(Some(ResourceType::Energy))
                > 110000
        {
            if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
                let _ = creep.ITwithdraw(storage, ResourceType::Energy, None);

                return;
            } else {
                let _ = creep.ITtransfer(nuker, ResourceType::Energy, None);

                return;
            }
        }
    }
    // end nuker stuffs.

    // Transfer what we didnt use.
    let _ = creep.ITtransfer(storage, ResourceType::Energy, None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn check_pos(creep: &Creep, memory: &mut ScreepsMemory, room_cache: &mut CachedRoom) -> bool {
    let wanted_pos = room_cache.storage_center;
    let pos = creep.pos();

    if let Some(wanted_pos) = wanted_pos {
        let wanted_pos = Position::new(wanted_pos.x, wanted_pos.y, room_cache.room.name());
        if pos != wanted_pos {
            creep.bsay("🚚 POS", false);

            creep.better_move_to(memory, room_cache, wanted_pos, 0, MoveOptions::default());
            return true;
        }
    }

    false
}
