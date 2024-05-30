#![allow(dead_code)]

use screeps::{Creep, game, Position, RoomCoordinate};

use crate::{memory::CreepMemory, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions};

pub fn scout(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut RoomCache) {
    let creep_room = creep.room().unwrap();
    let exits = game::map::describe_exits(creep_room.name());

    let exit_to_use = exits.values().next().unwrap();
    creep.better_move_to(creepmem, cache, Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(25).unwrap(), exit_to_use), 25);

}