use std::collections::HashMap;

use rand::prelude::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use screeps::{find, game, Creep, HasPosition, Position, RoomPosition, SharedCreepProperties};

use crate::combat::rank_room;
use crate::{
    memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    let _ = creep.notify_when_attacked(false);

    if !memory
        .scouted_rooms
        .contains_key(&creep.room().unwrap().name())
    {
        let _ = creep.say("🔍", true);
        rank_room::rank_room(&creep.room().unwrap(), memory, cache);
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    if let Some(scout_target) = creep_memory.scout_target {
        let scout_target = RoomPosition::new(25, 25, scout_target);

        if creep.room().unwrap().name() == scout_target.room_name() {
            creep_memory.scout_target = None;
            run_creep(creep, memory, cache);
        } else {
            let _ = creep.say("🚚", false);
            creep.better_move_to(
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cache,
                scout_target.pos(),
                23,
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
            let mut rng = StdRng::seed_from_u64(game::time() as u64);
            exits.shuffle(&mut rng);
            exits.first().unwrap()
        } else {
            let mut rng = StdRng::seed_from_u64(game::time() as u64);
            exit_clone.choose(&mut rng).unwrap()
        };

        let pos = RoomPosition::new(25, 25, *exit);

        let _ = creep.say(&format!("👁️ {}", pos.room_name()), true);

        creep.better_move_to(
            memory.creeps.get_mut(&creep.name()).unwrap(),
            cache,
            pos.pos(),
            23,
        );
        memory.creeps.get_mut(&creep.name()).unwrap().scout_target = Some(*exit);
    }
}
