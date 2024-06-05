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
        rank_room::rank_room(&creep.room().unwrap(), memory, cache);
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    if let Some(scout_target) = creep_memory.scout_target {
        let scout_target = RoomPosition::new(25, 25, scout_target);

        if creep.room().unwrap().name() == scout_target.room_name() {
            creep_memory.scout_target = None;
        } else {
            creep.better_move_to(
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cache,
                scout_target.pos(),
                24,
            );
        }
    } else {
        let exits = game::map::describe_exits(creep.room().unwrap().name());
        let mut exits = exits.values().collect::<Vec<_>>();

        let _ = creep.say("🚪", false);
        let mut seedable = StdRng::seed_from_u64(game::time().into());
        exits.shuffle(&mut seedable);

        let exit = exits.first().unwrap();

        let pos = RoomPosition::new(25, 25, *exit);

        creep.better_move_to(
            memory.creeps.get_mut(&creep.name()).unwrap(),
            cache,
            pos.pos(),
            24,
        );
        memory.creeps.get_mut(&creep.name()).unwrap().scout_target = Some(*exit);
    }
}
