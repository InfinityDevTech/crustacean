use std::u32;

use screeps::{find, game, Creep, HasPosition, ResourceType, SharedCreepProperties};

use crate::{memory::{Role, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::RoomCache, traits::creep::CreepExtensions};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_scorer(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if let Some(flag) = game::flags().get("depositScore".to_string()) {
        //if flag.pos().get_range_to(creep.pos()) < 5 && creep.store().get_used_capacity(Some(ResourceType::Score)) == 0 {
        //    creep.suicide();
        //}
    }

    if creep.store().get_used_capacity(Some(ResourceType::Score)) == 0 {
        let owning_cache = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

        creep.bsay("NO-SCOR", false);

        if let Some(storage) = &owning_cache.structures.storage {
            if creep.pos().get_range_to(storage.pos()) <= 1 {
                if storage.store().get_used_capacity(Some(ResourceType::Score)) == 0 {
                    creep_memory.role = Role::Hauler;

                    return;
                }

                creep.withdraw(storage, ResourceType::Score, None);
            } else {
                if creep.ticks_to_live().unwrap_or(u32::MAX) <= creep.pos().get_range_to(storage.pos()) {
                    creep.suicide();
                }

                creep.bsay("MV-STOR", false);

                let pos = storage.pos().clone();
                creep.better_move_to(memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), pos, 1, MoveOptions::default().avoid_hostile_rooms(true));

                return;
            }
        }
    }

    if let Some(flag) = game::flags().get("depositScore".to_string()) {
        if creep.pos().get_range_to(flag.pos()) > 1 {
            creep.better_move_to(memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), flag.pos(), 1, MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(true).avoid_hostile_remotes(true).path_age(15));
        } else {
            let sc = creep.room().unwrap().find(find::SCORE_COLLECTORS, None);

            if let Some(sc) = sc.first() {
                creep.transfer(sc, ResourceType::Score, Some(creep.store().get_used_capacity(Some(ResourceType::Score))));
            }
        }
    }
}