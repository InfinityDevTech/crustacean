use rand::prelude::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};
use screeps::{find, game, Creep, HasPosition, Position, SharedCreepProperties};

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
        let scout_target =
            Position::new(scout_target.x, scout_target.y, creep.room().unwrap().name());

        if creep.pos().get_range_to(scout_target) == 0 {
            creep_memory.scout_target = None;
        } else {
            creep.better_move_to(
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cache,
                scout_target,
                1,
            );
        }
    } else {
        let mut exits = creep.room().unwrap().find(find::EXIT, None);

        let _ = creep.say("🚪", false);
        let mut seedable = StdRng::seed_from_u64(game::time().into());
        exits.shuffle(&mut seedable);

        let exit = exits.first().unwrap();
        creep.better_move_to(
            memory.creeps.get_mut(&creep.name()).unwrap(),
            cache,
            exit.pos(),
            1,
        );
        memory.creeps.get_mut(&creep.name()).unwrap().scout_target = Some(exit.pos().xy());
    }
}
