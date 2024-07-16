use screeps::{Creep, HasPosition, Position, RoomCoordinate, SharedCreepProperties};

use crate::{memory::{Role, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking}};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_remotedefender(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if creep.hits() < creep.hits_max() {
        let _ = creep.ITheal(creep);
    }

    if let Some(target_room) = creep_memory.target_room {
        if creep.room().unwrap().name() != target_room {
            let position = Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(25).unwrap(), target_room);

            creep.better_move_to(memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), position, 23, Default::default());
        } else if let Some(cache) = cache.rooms.get_mut(&creep.room().unwrap().name()) {
            let hostile_creeps = &cache.creeps.enemy_creeps_with_attack;

            if !hostile_creeps.is_empty() {
                let target = hostile_creeps.first().unwrap();

                if creep.pos().get_range_to(target.pos()) >= 4 {
                    creep.better_move_to(memory, cache, target.pos(), 1, MoveOptions::default().path_age(2));
                } else {
                    let range = creep.pos().get_range_to(target.pos());

                    if range > 1 && range <= 3 {
                        let _ = creep.ITranged_attack(target);
                    } else if range <= 1 {
                        let _ = creep.ITranged_mass_attack();
                    }

                    creep.bsay("🔫", false);
                    creep.better_move_to(memory, cache, target.pos(), 1, MoveOptions::default().path_age(2));
                }
            } else {
                creep_memory.role = Role::Recycler;
            }
        }
    }
}