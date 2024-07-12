use log::info;
use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{game, Color, Creep, HasPosition, OwnedStructureProperties, Position, RoomCoordinate, SharedCreepProperties};

use crate::{
    config, memory::{Role, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking}, utils::get_my_username
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

    info!("Claimer: {} - {:?}", creep.name(), creep_memory);

    if creep.ticks_to_live() < Some(100) || creep_memory.target_room.is_none() {
        creep_memory.role = Role::Recycler;
    }

    let current_room = creep.room().unwrap();

    if current_room.name() != creep_memory.target_room.unwrap() {
        let pos = Position::new(unsafe {
            RoomCoordinate::unchecked_new(25)
        }, unsafe {
            RoomCoordinate::unchecked_new(25)
        }, creep_memory.target_room.unwrap());

        creep.better_move_to(memory, room_cache, pos, 23, MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(true));
    } else {
        let controller = current_room.controller().unwrap();

        if controller.my() {
            creep.bsay("🏳️", true);
            return;
        } else if creep.pos().is_near_to(controller.pos()) {
            let _ = creep.ITclaim_controller(&controller);
        } else {
            creep.better_move_to(memory, room_cache, controller.pos(), 1, MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(true));
        }
    }
}
