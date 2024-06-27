use rand::prelude::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use screeps::{game, memory, Creep, HasPosition, RoomName, RoomPosition, SharedCreepProperties};

use crate::movement::move_target::MoveOptions;
use crate::{
    memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions,
};

// TODO: Make these guys more top down.
// as in, the room designates the scout target, and the scout goes to it.
// So its loads more dynamic.
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_scout(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    let _ = creep.notify_when_attacked(false);
    let _ = creep.say("🔍 😛", true);

    let cached_room = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if let Some(scout_target) = creep_memory.scout_target {
        let scout_target = RoomPosition::new(25, 25, scout_target);

        if creep.room().unwrap().name() == scout_target.room_name() {
            if creep.pos().get_range_to(scout_target.pos()) <= 23 {
                let _ = creep.say("🔍 🏠", true);

                creep_memory.scout_target = None;

                run_scout(creep, memory, cache);
            } else {
                creep.better_move_to(
                    memory.creeps.get_mut(&creep.name()).unwrap(),
                    cached_room,
                    scout_target.pos(),
                    23,
                    MoveOptions::default().avoid_enemies(true)
                );
            }
        } else {
            let _ = creep.say("🔍 😛", true);
            creep.better_move_to(
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cached_room,
                scout_target.pos(),
                23,
                MoveOptions::default().avoid_enemies(true)
            );
        }
    } else {
        let exits = game::map::describe_exits(creep.room().unwrap().name());
        let mut exits = exits.values().collect::<Vec<_>>();

        let mut exit_clone = exits.clone();

        let _ = creep.say("🚚", false);

        for exit in exits.clone() {
            let existing_data = memory.scouted_rooms.get(&exit);


            if (existing_data.is_some() && existing_data.unwrap().last_scouted + 3000 < game::time()) || memory.rooms.contains_key(&exit) || memory.remote_rooms.contains_key(&exit) {
                exit_clone.retain(|x| *x != exit);
            }
        }

        let exit = if exit_clone.is_empty() {
            let mut top_scorer = None;
            let mut top_scorer_age = u32::MAX;

            for exit in exits.clone() {
                let room = memory.scouted_rooms.get(&exit);

                // If we scouted it, check if it's the oldest
                // if we havent, go to it.
                if let Some(room) = room {
                    let last_scout = room.last_scouted;

                    if last_scout > top_scorer_age {
                        top_scorer = Some(exit);
                        top_scorer_age = last_scout;
                    }
                } else {
                    top_scorer = Some(exit);
                    top_scorer_age = u32::MAX;
                }
            }

            if top_scorer.is_none() {
                let mut rng = StdRng::seed_from_u64(game::time() as u64);
                exits.choose(&mut rng).unwrap()
            } else {
                &top_scorer.unwrap()
            }
        } else {
            let mut rng = StdRng::seed_from_u64(game::time() as u64);
            exit_clone.choose(&mut rng).unwrap()
        };

        let pos = RoomPosition::new(25, 25, *exit);

        let _ = creep.say(&format!("👁️ {}", pos.room_name()), true);

        creep.better_move_to(
            memory.creeps.get_mut(&creep.name()).unwrap(),
            cached_room,
            pos.pos(),
            23,
            MoveOptions::default().avoid_enemies(true)
        );
        memory.creeps.get_mut(&creep.name()).unwrap().scout_target = Some(*exit);
    }
}
