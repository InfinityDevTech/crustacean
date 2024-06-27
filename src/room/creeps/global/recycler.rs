use log::info;
use screeps::{game, Creep, HasPosition, Part, SharedCreepProperties};

use crate::{
    memory::ScreepsMemory, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_recycler(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    // If the creep is a spud and cant move, suicide
    // Because its wasting my damn space and time.
    let parts = creep
        .body()
        .iter()
        .filter(|p| p.part() == Part::Move && p.hits() > 0)
        .count();
    if parts == 0 {
        // If we reach this point, the creep is lacking something
        // to be able to recycle properly. So we just commit suicide
        // Duh...
        let _ = creep.say("AAHHHHHHHHH", true);
        let _ = creep.suicide();
    }


    let owning_room = memory.creeps.get(&creep.name());
    if owning_room.is_none() {
        return;
    }

    let owning_room = owning_room.unwrap().owning_room;
    if let Some(room) = game::rooms().get(owning_room) {
        cache.create_if_not_exists(&room, memory, None);
    }

    if let Some(creep_memory) = memory.creeps.get_mut(&creep.name()) {
        if let Some(owning_room) = game::rooms().get(creep_memory.owning_room) {
            if let Some(room_cache) = cache.rooms.get_mut(&owning_room.name()) {
                if let Some(spawn) = room_cache.structures.spawns.values().next() {
                    let current_pos = creep.pos();
                    let spawn_pos = spawn.pos();

                    if current_pos.is_near_to(spawn_pos) {
                        let _ = spawn.recycle_creep(creep);

                        return;
                    } else {
                        creep.better_move_to(creep_memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), spawn_pos, 1, MoveOptions::default().avoid_enemies(true));
                        return;
                    }
                }
            }
        }
    }

    // If we reach this point, the creep is lacking something
    // to be able to recycle properly. So we just commit suicide
    // Duh...
    let _ = creep.say("AAHHHHHHHHH", true);
    let _ = creep.suicide();
}
