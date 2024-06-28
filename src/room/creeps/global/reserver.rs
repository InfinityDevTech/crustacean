use screeps::{game, Creep, HasPosition, Position, RoomCoordinate, SharedCreepProperties};

use crate::{memory::{Role, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions};

use super::recycler::run_recycler;

pub fn run_reserver(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    let current_room = creep.room().unwrap();
    if let Some(target_room) = creep_memory.target_room {

        if let Some(task) = memory.goals.room_reservation.get_mut(&target_room) {
            if game::time() % 10 == 0 && !task.creeps_assigned.contains(&creep.name()) {
                task.creeps_assigned.push(creep.name());
            }
        }

        if current_room.name() != target_room {
            let position = Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(25).unwrap(), target_room);
            creep.better_move_to(creep_memory, cache.rooms.get_mut(&current_room.name()).unwrap(), position, 23, MoveOptions::default().avoid_enemies(true));
        } else {
            let controller = current_room.controller().unwrap();

            if creep.pos().is_near_to(controller.pos()) {
                let _ = creep.reserve_controller(&controller);
            } else {
                creep.better_move_to(creep_memory, cache.rooms.get_mut(&current_room.name()).unwrap(), controller.pos(), 1, MoveOptions::default().avoid_enemies(true));
            }
        }
    } else {
        creep_memory.role = Role::Recycler;
        run_recycler(creep, memory, cache);
    }
}