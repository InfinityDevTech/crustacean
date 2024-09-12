use screeps::{game, Creep, HasPosition, OwnedStructureProperties, Position, RoomCoordinate, SharedCreepProperties};

use crate::{
    memory::ScreepsMemory, movement::move_target::MoveOptions, room::{cache::RoomCache, creeps::local::upgrader::sign_controller}, traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking}
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_claimer(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let room_cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    if creep.spawning() {
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name());
    if creep_memory.is_none() {
        return;
    }
    let creep_memory = creep_memory.unwrap();

    let current_room = creep.room().unwrap();

    if current_room.name() != creep_memory.target_room.unwrap() {
        let pos = Position::new(unsafe {
            RoomCoordinate::unchecked_new(25)
        }, unsafe {
            RoomCoordinate::unchecked_new(25)
        }, creep_memory.target_room.unwrap());

        if let Some(flag) = game::flags().get("forceClaimerPath".to_string()) {
            creep.better_move_to(memory, room_cache, flag.pos(), 1, MoveOptions::default().visualize_path(true).ignore_cache(true).path_age(15));

            return;
        }

        if let Some(scouting_data) = memory.scouted_rooms.get(&creep_memory.target_room.unwrap()) {
            if scouting_data.controller.is_some() && creep_memory.scout_target.is_some() {
                let pos = Position::new(scouting_data.controller.unwrap().x, scouting_data.controller.unwrap().y, creep_memory.scout_target.unwrap());

                creep.better_move_to(memory, room_cache, pos, 1, MoveOptions::default().visualize_path(true).ignore_cache(true).path_age(15));

                return;
            }
        }

        creep.better_move_to(memory, room_cache, pos, 23, MoveOptions::default().visualize_path(true).ignore_cache(true).path_age(15));
    } else {
        let controller = current_room.controller().unwrap();

        if controller.my() {
            let _ = sign_controller(creep, memory, cache);
            creep.bsay("🏳️", true);
        } else if creep.pos().is_near_to(controller.pos()) {
            let _ = creep.ITclaim_controller(&controller);
        } else {
            creep.better_move_to(memory, room_cache, controller.pos(), 1, MoveOptions::default().avoid_enemies(true).visualize_path(true).ignore_cache(true));
        }
    }
}
