use screeps::{Creep, HasPosition, SharedCreepProperties};

use crate::{
    memory::ScreepsMemory,
    room::cache::{CachedRoom, RoomCache},
    traits::creep::CreepExtensions,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_mineralminer(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let room_cache = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if room_cache.structures.containers().mineral.is_none() {
        creep.bsay("NO CONT", false);
        return;
    }

    if let Some(storage) = &room_cache.structures.storage {
        if let Some(mineral) = &room_cache.resources.mineral {
            if storage.store().get_used_capacity(Some(mineral.mineral_type())) >= 50000 {
                creep.bsay("STORAGE FULL", false);
                return;
            }
        }
    } else {
        return;
    }

    if let Some(mineral_container) = &room_cache.structures.containers().mineral {
        if creep.pos() != mineral_container.pos() {
            creep.better_move_to(
                memory,
                room_cache,
                mineral_container.pos(),
                1,
                Default::default(),
            );
        }
    }

    if let Some(mineral) = &room_cache.resources.mineral.clone() {
        creep.set_working_area(room_cache, mineral.pos(), 1);
        if creep.pos().is_near_to(mineral.pos()) {
            let _ = creep.harvest(mineral);

            creep.bsay("⛏️", false);

            deposit_energy(creep, memory, room_cache);
        } else {
            creep.bsay("🚚 MINERAL", false);
            creep.better_move_to(memory, room_cache, mineral.pos(), 1, Default::default());
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn deposit_energy(creep: &Creep, memory: &mut ScreepsMemory, room_cache: &mut CachedRoom) {
    if creep.store().get_used_capacity(None) == 0 {
        return;
    }

    if let Some(mineral) = &room_cache.resources.mineral {
        if let Some(mineral_container) = &room_cache.structures.containers().mineral {
            if creep.pos() == mineral_container.pos() {
                return;
            }


            if creep.pos().is_near_to(mineral_container.pos()) {
                let _ = creep.transfer(mineral_container, mineral.mineral_type(), None);
            } else {
                creep.better_move_to(
                    memory,
                    room_cache,
                    mineral_container.pos(),
                    1,
                    Default::default(),
                );
            }
        }
    }
}
