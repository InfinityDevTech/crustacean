use screeps::{game, Creep, HasPosition, MaybeHasId, ObjectId, Position, RoomCoordinate, RoomXY, SharedCreepProperties};

use crate::{heap, memory::ScreepsMemory, traits::intents_tracking::CreepExtensionsTracking};

use super::CachedRoom;

pub mod simple_solver;
pub mod advanced_solver;
pub mod traffic_cache;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_movement(room_cache: &mut CachedRoom, memory: &mut ScreepsMemory) -> u32 {
    let pre_traffic_cpu = game::cpu::get_used();

    // Watch this, its a hack for some bugs. This is a temporary fix
    // Haulers would have no task, and block the path, I might have them move a random dir.
    // TODO: Fix this
    if !memory.rooms.contains_key(&room_cache.room.name()) && !memory.remote_rooms.contains_key(&room_cache.room.name()) {
        run_non_room_traffic(room_cache);

        return 0;
    }

    room_cache.traffic.movement_map.clear();
    let mut creeps_with_movement_intent = Vec::new();

    let creep_names: Vec<String> = room_cache.creeps.creeps_in_room.keys().cloned().collect();
    // Just save some CPU, not much, but CPU is CPU
    if creep_names.is_empty() { return 0; }

    assign_coordinates(&creep_names, room_cache, &mut creeps_with_movement_intent);
    if creeps_with_movement_intent.is_empty() { return 0; }

    if memory.rooms.contains_key(&room_cache.room.name()) {
        advanced_solver::solve_traffic_advanced(&creeps_with_movement_intent, room_cache);
    } else {
        simple_solver::solve_traffic_simple(&creeps_with_movement_intent, room_cache);
    }

    move_creeps(&creep_names, room_cache);

    let post_traffic_cpu = game::cpu::get_used();
    room_cache.stats.cpu_traffic = post_traffic_cpu - pre_traffic_cpu;

    room_cache.traffic.move_intents as u32
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn run_non_room_traffic(room_cache: &mut CachedRoom) -> u32 {
    let mut i = 0;

    for (creep, matched_coord) in room_cache.traffic.intended_move.clone() {
        let creep = game::get_object_by_id_typed(&creep).unwrap();

        if matched_coord == creep.pos().xy() {
            continue;
        }
        let x = RoomCoordinate::new(matched_coord.x.u8());
        let y = RoomCoordinate::new(matched_coord.y.u8());

        if x.is_err() || y.is_err() {
            continue;
        }

        let position = Position::new(x.unwrap(), y.unwrap(), creep.room().unwrap().name());

        let direction = creep.pos().get_direction_to(position).unwrap();
        let res = creep.ITmove_direction(direction);

        if res.is_ok() {
            i += 1;
        }

        if let Some(heap_creep) = heap().creeps.lock().unwrap().get_mut(&creep.name()) {
            heap_creep.update_position(&creep)
        }
    }

    i
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn move_creeps(creep_names: &Vec<String>, room_cache: &mut CachedRoom) {
    for creep_name in creep_names {
        let creep = game::creeps().get(creep_name.to_string()).unwrap();
        let matched_coord = room_cache.traffic.matched_coord.get(&creep.try_id().unwrap());

        if matched_coord.is_none() || *matched_coord.unwrap() == creep.pos().xy() {
            continue;
        }
        let x = RoomCoordinate::new(matched_coord.unwrap().x.u8());
        let y = RoomCoordinate::new(matched_coord.unwrap().y.u8());

        if x.is_err() || y.is_err() {
            continue;
        }

        let position = Position::new(x.unwrap(), y.unwrap(), creep.room().unwrap().name());

        let direction = creep.pos().get_direction_to(position).unwrap();
        let res = creep.ITmove_direction(direction);

        if res.is_err() {
            let _err = res.unwrap_err();
        } else {
            room_cache.traffic.move_intents += 1;

            if let Some(heap_creep) = heap().creeps.lock().unwrap().get_mut(&creep.name()) {
                heap_creep.update_position(&creep)
            }
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn assign_coordinates(creep_names: &Vec<String>, room_cache: &mut CachedRoom, creeps_with_movement_intent: &mut Vec<ObjectId<Creep>>) {
    for creep_name in creep_names {
        let creep = game::creeps().get(creep_name.to_string()).unwrap();

        assign_creep_to_coordinate(&creep, room_cache, creep.pos().into());

        if room_cache.traffic.intended_move.contains_key(&creep.try_id().unwrap()) {
            creeps_with_movement_intent.push(creep.try_id().unwrap());
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn assign_creep_to_coordinate(creep: &Creep, room_cache: &mut CachedRoom, coord: RoomXY) {
    let packed_coord = coord;

    room_cache.traffic.matched_coord.insert(creep.try_id().unwrap(), packed_coord);
    room_cache.traffic.movement_map.insert(packed_coord, creep.try_id().unwrap());
}