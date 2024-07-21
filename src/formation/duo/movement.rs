use screeps::{Creep, HasPosition, Part, Position, SharedCreepProperties};

use crate::{memory::ScreepsMemory, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions};

use super::duo_utils::{get_attacker, get_healer};

pub fn move_duo(creeps: Vec<Creep>, memory: &mut ScreepsMemory, cache: &mut RoomCache, dest: Position, range: u16, move_options: Option<MoveOptions>) {
    let healer = get_healer(&creeps);
    let attacker = get_attacker(&creeps);

    if healer.is_none() || attacker.is_none() {
        return;
    }

    let healer = healer.unwrap();
    let attacker = attacker.unwrap();

    if !can_duo_move(&creeps) {
        return;
    }

    let healer_pos = healer.pos();
    let attacker_pos = attacker.pos();

    form_duo(&creeps, memory, cache, dest, range, move_options);
    if !is_duo_formed(&creeps) {
        return;
    }

    attacker.better_move_to(memory, cache.rooms.get_mut(&attacker.room().unwrap().name()).unwrap(), dest, range, move_options.unwrap_or_default());

    let _ = healer.move_direction(healer_pos.get_direction_to(attacker_pos).unwrap());
}

fn can_duo_move(creeps: &Vec<Creep>) -> bool {
    for creep in creeps {
        if creep.tired() {
            return false;
        }
    }

    true
}

fn form_duo(creeps: &Vec<Creep>, memory: &mut ScreepsMemory, cache: &mut RoomCache, dest: Position, range: u16, move_options: Option<MoveOptions>) {
    let healer = get_healer(creeps).unwrap();
    let attacker = get_attacker(creeps).unwrap();

    let healer_room = healer.room().unwrap();
    let attacker_room = attacker.room().unwrap();

    if is_duo_formed(creeps) {
        return;
    }

    if healer_room.name() != attacker_room.name() {
        if attacker.pos().is_room_edge() {
            // This is to move it off the edge, so the healer can come in.
            attacker.better_move_to(memory, cache.rooms.get_mut(&attacker.room().unwrap().name()).unwrap(), dest, range, move_options.unwrap_or_default());
            return;
        }
        healer.better_move_to(memory, cache.rooms.get_mut(&healer.room().unwrap().name()).unwrap(), attacker.pos(), 1, MoveOptions::default());
        return;
    }

    healer.better_move_to(memory, cache.rooms.get_mut(&healer.room().unwrap().name()).unwrap(), attacker.pos(), 1, MoveOptions::default());
}

fn is_duo_formed(creeps: &Vec<Creep>) -> bool {
    for top_creep in creeps.clone() {
        for bottom_creep in creeps {
            if !top_creep.pos().is_near_to(bottom_creep.pos()) {
                return false;
            }
        }
    }

    true
}