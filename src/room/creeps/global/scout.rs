use rand::prelude::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use screeps::game::map::RoomStatus;
use screeps::{game, Creep, HasPosition, RoomPosition, SharedCreepProperties};

use crate::movement::move_target::MoveOptions;
use crate::room::creeps::local::upgrader::sign_controller;
use crate::{
    memory::ScreepsMemory, room::cache::RoomCache, traits::creep::CreepExtensions,
};

// TODO: Make these guys more top down.
// as in, the room designates the scout target, and the scout goes to it.
// So its loads more dynamic.
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_scout(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        creep.bsay("😴", false);
        return;
    }

    let _ = creep.notify_when_attacked(false);
    creep.bsay("🔍 😛", true);

    let cached_room = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if let Some(scout_target) = creep_memory.scout_target {
        let scout_target = RoomPosition::new(25, 25, scout_target);

        if creep.room().unwrap().name() == scout_target.room_name() {
            if sign_controller(creep, memory, cache) {
                return;
            }

            let cached_room = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();
            let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

            if creep.pos().get_range_to(scout_target.pos()) <= 23 {
                creep.bsay("🔍 🏠", true);

                creep_memory.scout_target = None;

                run_scout(creep, memory, cache);
            } else {
                creep.better_move_to(
                    memory,
                    cached_room,
                    scout_target.pos(),
                    23,
                    MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(false)
                );
            }
        } else {
            creep.bsay("🔍 😛", true);
            creep.better_move_to(
                memory,
                cached_room,
                scout_target.pos(),
                23,
                MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(false)
            );
        }
    } else {
        let exits = game::map::describe_exits(creep.room().unwrap().name());
        let exits = exits.values().collect::<Vec<_>>();

        let mut not_scouted = Vec::new();
        let mut out_of_date_scouted = Vec::new();
        let mut last_scouted = Vec::new();

        for exit in exits {
            let room_status = game::map::get_room_status(exit);

            if room_status.is_none() || room_status.unwrap().status() != RoomStatus::Normal || memory.rooms.contains_key(&exit) {
                continue;
            }

            if !memory.scouted_rooms.contains_key(&exit) {
                not_scouted.push(exit);
            } else {
                let last_scouted_time = memory.scouted_rooms.get(&exit).unwrap();

                if game::time() - last_scouted_time.last_scouted > 3000 {
                    out_of_date_scouted.push((exit, last_scouted_time.last_scouted));
                } else {
                    last_scouted.push((exit, last_scouted_time.last_scouted));
                }
            }
        }

        out_of_date_scouted.sort_by_key(|x| x.1);
        last_scouted.sort_by_key(|x| x.1);

        let mut exit = if game::flags().get("force_scout".to_string()).is_some() {
            &game::flags().get("force_scout".to_string()).unwrap().pos().room_name()
        } else if let Some(exit) = not_scouted.first() {
            exit
        } else if let Some(exit) = out_of_date_scouted.first() {
            &exit.0
        } else if let Some(exit) = last_scouted.first() {
            &exit.0
        } else {
            return;
        };

        let pos = RoomPosition::new(25, 25, *exit);

        creep.bsay(&format!("👁️ {}", pos.room_name()), true);

        creep.better_move_to(
            memory,
            cached_room,
            pos.pos(),
            23,
            MoveOptions::default().avoid_enemies(true)
        );
        memory.creeps.get_mut(&creep.name()).unwrap().scout_target = Some(*exit);
    }
}
