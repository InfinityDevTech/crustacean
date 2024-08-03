#![allow(non_snake_case)]use std::collections::HashMap;

use screeps::{
    game, Creep, HasPosition, MaybeHasId, ObjectId, Position, RoomCoordinate, RoomXY, SharedCreepProperties
};

use super::CachedRoom;
use crate::{heap, memory::ScreepsMemory, traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking}};

#[derive(Debug, Clone)]
pub struct TrafficCache {
    pub matched_coord: HashMap<ObjectId<Creep>, RoomXY>,
    pub intended_move: HashMap<ObjectId<Creep>, RoomXY>,

    pub movement_map: HashMap<RoomXY, ObjectId<Creep>>,

    pub cached_ops: HashMap<ObjectId<Creep>, Vec<RoomXY>>,
    pub move_intents: u8,
}

impl TrafficCache {
    pub fn new() -> Self {
        Self {
            matched_coord: HashMap::new(),
            intended_move: HashMap::new(),
            movement_map: HashMap::new(),
            cached_ops: HashMap::new(),
            move_intents: 0,
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_movement(room_cache: &mut CachedRoom, memory: &mut ScreepsMemory) {
    let pre_traffic_cpu = game::cpu::get_used();

    // Watch this, its a hack for some bugs. This is a temporary fix
    // Haulers would have no task, and block the path, I might have them move a random dir.
    // TODO: Fix this
    if !memory.rooms.contains_key(&room_cache.room_name) && !memory.remote_rooms.contains_key(&room_cache.room_name) {
        run_non_room_traffic(room_cache);

        return;
    }

    room_cache.traffic.movement_map.clear();
    let mut creeps_with_movement_intent = Vec::new();

    let creep_names: Vec<String> = room_cache.creeps.creeps_in_room.keys().cloned().collect();

    // Just save some CPU, not much, but CPU is CPU
    if creep_names.is_empty() { return; }

    assign_coordinates(&creep_names, room_cache, &mut creeps_with_movement_intent);

    if creeps_with_movement_intent.is_empty() {
        return;
    }

    let mut visited_creeps = Vec::new();

    solve_traffic(&creeps_with_movement_intent, room_cache, &mut visited_creeps);

    move_creeps(&creep_names, room_cache);

    let post_traffic_cpu = game::cpu::get_used();
    room_cache.stats.cpu_traffic = post_traffic_cpu - pre_traffic_cpu;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn run_non_room_traffic(room_cache: &mut CachedRoom) {
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
        let _ = creep.ITmove_direction(direction);

        if let Some(heap_creep) = heap().creeps.lock().unwrap().get_mut(&creep.name()) {
            heap_creep.update_position(&creep)
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
fn solve_traffic(creeps_with_movement_intent: &Vec<ObjectId<Creep>>, room_cache: &mut CachedRoom, visited_creeps: &mut Vec<ObjectId<Creep>>) {
    for creep_id in creeps_with_movement_intent {
        let creep = game::get_object_by_id_typed(creep_id).unwrap();
        if room_cache.traffic.matched_coord.get(creep_id) == room_cache.traffic.intended_move.get(creep_id) {
            continue;
        }

        if room_cache.traffic.matched_coord.contains_key(creep_id) {
            room_cache.traffic.movement_map.remove(&room_cache.traffic.matched_coord[creep_id]);
        }
        room_cache.traffic.matched_coord.remove(creep_id);

        if depth_first_searh(&creep, room_cache, visited_creeps, Some(0)) > 0 {
            continue;
        }

        assign_creep_to_coordinate(&creep, room_cache, creep.pos().xy());
    }
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
fn depth_first_searh(creep: &Creep, room_cache: &mut CachedRoom, visited_creeps: &mut Vec<ObjectId<Creep>>, score: Option<i32>) -> i32 {
    let mut score = score.unwrap_or(0);
    visited_creeps.push(creep.try_id().unwrap());

    let possible_moves = creep.get_possible_moves(room_cache);

    let mut empty_tiles = Vec::new();
    let mut occupied_tiles = Vec::new();

    for coord in possible_moves {
        if room_cache.traffic.movement_map.contains_key(&coord) {
            occupied_tiles.push(coord);
        } else {
            empty_tiles.push(coord);
        }
    }

    let mut combined = empty_tiles.clone();
    combined.extend(occupied_tiles.clone());

    for coord in combined {
        if room_cache.traffic.intended_move.contains_key(&creep.try_id().unwrap()) && *room_cache.traffic.intended_move.get(&creep.try_id().unwrap()).unwrap() == coord {
            score += 1;
        }

        let occupying = room_cache.traffic.movement_map.get(&coord);

        if occupying.is_none() {
            if score > 0 {
                assign_creep_to_coordinate(creep, room_cache, coord)
            }
            return score;
        }

        if !visited_creeps.contains(occupying.unwrap()) {
            if room_cache.traffic.intended_move.contains_key(occupying.unwrap()) && *room_cache.traffic.intended_move.get(occupying.unwrap()).unwrap() == coord {
                score -= 1;
            }

            let result = depth_first_searh(&game::get_object_by_id_typed(occupying.unwrap()).unwrap(), room_cache, visited_creeps, Some(score));

            if result > 0 {
                assign_creep_to_coordinate(creep, room_cache, coord);
                return result;
            }
        }
    }

    i32::MIN
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn assign_creep_to_coordinate(creep: &Creep, room_cache: &mut CachedRoom, coord: RoomXY) {
    let packed_coord = coord;

    room_cache.traffic.matched_coord.insert(creep.try_id().unwrap(), packed_coord);
    room_cache.traffic.movement_map.insert(packed_coord, creep.try_id().unwrap());
}