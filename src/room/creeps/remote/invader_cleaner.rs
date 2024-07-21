use screeps::{game, Creep, HasPosition, SharedCreepProperties};

use crate::{memory::ScreepsMemory, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::{creep::CreepExtensions, room::RoomExtensions}};

pub fn run_invadercleaner(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let creep_room = creep.room().unwrap();

    if creep_memory.target_room.is_none() {
        creep.bsay("kurt kob", true);
        return;
    }

    if creep_room.name() != creep_memory.target_room.unwrap() {
        creep.better_move_to(
            memory,
            cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
            creep_room.center_pos(),
            23,
            MoveOptions::default(),
        );
        return;
    }

    let target_room = cache.rooms.get_mut(&creep_memory.target_room.unwrap()).unwrap();
    let invader_core = target_room.structures.invader_core.as_ref().unwrap();

    if creep.pos().get_range_to(invader_core.pos()) > 1 {
        let pos = invader_core.pos();

        creep.bsay("🚚 - ICORE", false);

        creep.better_move_to(
            memory,
            cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
            pos,
            1,
            MoveOptions::default(),
        );
    } else {
        if game::time() % 2 == 0 {
            creep.bsay("FUCK", false);
        } else {
            creep.bsay("YOU", false);
        }

        let _ = creep.attack(invader_core);
    }
}