use rand::prelude::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use screeps::{game, Creep, HasPosition, RoomPosition, SharedCreepProperties};

use crate::movement::move_target::MoveOptions;
use crate::{
    memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions,
};

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
            creep_memory.scout_target = None;
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
            if memory.scouted_rooms.contains_key(&exit) || memory.rooms.contains_key(&exit) {
                exit_clone.retain(|x| *x != exit);
            }
        }

        let exit = if exit_clone.is_empty() {

            exits.sort_by(|a, b| {
                let a = memory.scouted_rooms.get(a);
                let b = memory.scouted_rooms.get(b);

                if a.is_none() || b.is_none() {
                    std::cmp::Ordering::Equal
                } else {
                    let a = a.unwrap();
                    let b = b.unwrap();

                    a.last_scouted.cmp(&b.last_scouted)
                }
            });
            exits.first().unwrap()
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
