use std::collections::HashMap;

use log::info;
use screeps::{
    game,
    pathfinder::{self, MultiRoomCostResult, SearchOptions},
    CircleStyle, HasPosition, LocalCostMatrix, Position, Room, RoomCoordinate, RoomName, RoomXY,
    StructureProperties, StructureType,
};

use crate::{
    compression::{decode_pos_list, encode_pos_list}, constants::{SWAMP_MASK, WALKABLE_STRUCTURES, WALL_MASK}, memory::ScreepsMemory, profiling::timing::PATHFIND_CPU, room::cache::RoomCache, traits::position::RoomXYExtensions
};

use super::construction::get_all_structure_plans;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn plan_main_room_roads(
    room: &Room,
    cache: &mut RoomCache,
    memory: &mut ScreepsMemory,
) -> HashMap<RoomName, String> {
    let mut room_path_destinations = Vec::new();

    if let Some(owning_cache) = cache.rooms.get(&room.name()) {
        if let Some(storage) = &owning_cache.structures.storage {
            room_path_destinations.push(storage.pos());
        }

        if let Some(controller) = &owning_cache.structures.controller {
            room_path_destinations.push(controller.pos());
        }

        if let Some(mineral) = &owning_cache.resources.mineral {
            room_path_destinations.push(mineral.pos());
        }

        for source in &owning_cache.resources.sources {
            room_path_destinations.push(source.source.pos());
        }
    }

    if let Some(owning_memory) = memory.rooms.get_mut(&room.name()) {
        let remotes = owning_memory.remotes.clone();

        for remote in remotes {
            if let Some(remote_cache) = cache.rooms.get(&remote) {
                for source in &remote_cache.resources.sources {
                    room_path_destinations.push(source.source.pos());
                }
            }
        }
    }

    let measure_pos = memory
        .rooms
        .get(&room.name())
        .unwrap()
        .storage_center
        .as_position(&room.name());

    let mut closest_to_furthest = room_path_destinations.clone();
    closest_to_furthest.sort_by_key(|pos| pos.get_range_to(measure_pos));

    let mut furthest_to_closest = closest_to_furthest.clone();
    furthest_to_closest.reverse();

    let p_c_to_f = path_roads_from_pos(cache, memory, measure_pos, closest_to_furthest);
    let p_f_to_c = path_roads_from_pos(cache, memory, measure_pos, furthest_to_closest);

    let total_c_to_f = count_total_roads(p_c_to_f.clone());
    let total_f_to_c = count_total_roads(p_f_to_c.clone());

    if total_c_to_f < total_f_to_c {
        encode_all_paths(p_c_to_f.clone())
    } else {
        encode_all_paths(p_f_to_c.clone())
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn encode_all_paths(paths: HashMap<RoomName, Vec<Position>>) -> HashMap<RoomName, String> {
    let mut encoded_paths = HashMap::new();

    for (room_name, path) in paths {
        let encoded = encode_pos_list(path);
        encoded_paths.insert(room_name, encoded);
    }

    encoded_paths
}

pub fn visualise_paths(paths: HashMap<RoomName, Vec<Position>>) {
    for (room_name, path) in paths {
        if let Some(game_room) = game::rooms().get(room_name) {
            let vis = game_room.visual();
            for pos in path {
                let x = pos.x().u8() as f32;
                let y = pos.y().u8() as f32;
                vis.circle(
                    x,
                    y,
                    Some(CircleStyle::default().fill("#ff0000").radius(0.3)),
                );
            }
        }
    }
}

pub fn get_all_positions_from_paths(paths: HashMap<RoomName, Vec<Position>>) -> Vec<Position> {
    let mut all_positions = Vec::new();

    for (_room_name, path) in paths {
        all_positions.extend(path);
    }

    all_positions
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn count_total_roads(paths: HashMap<RoomName, Vec<Position>>) -> usize {
    let mut total = 0;

    for (_room_name, path) in paths {
        total += path.len();
    }

    total
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_all_cached_road_positions(
    room_name: &RoomName,
    memory: &ScreepsMemory,
) -> HashMap<RoomName, Vec<Position>> {
    let mut rooms = HashMap::new();

    if let Some(room_memory) = memory.rooms.get(room_name) {
        for (room_name, encoded_pos) in &room_memory.planned_paths {
            let positions = decode_pos_list(encoded_pos.to_string());

            rooms.insert(room_name.clone(), positions);
        }
    }

    rooms
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn path_roads_from_pos(
    cache: &mut RoomCache,
    memory: &ScreepsMemory,
    source: Position,
    destinations: Vec<Position>,
) -> HashMap<RoomName, Vec<Position>> {
    let mut new_roads = HashMap::new();
    let mut paths = HashMap::new();

    let all = get_all_cached_road_positions(&source.room_name(), memory);

    for destination in destinations {
        let pre_pathfind_cpu = game::cpu::get_used();

        let result = pathfinder::search(
            source,
            destination,
            1,
            Some(SearchOptions::new(|room_name| {
                room_callback(&room_name, cache, memory, new_roads.clone(), all.clone())
            })),
        );

        *PATHFIND_CPU.lock().unwrap() += game::cpu::get_used() - pre_pathfind_cpu;

        let path = result.path();

        for pos in path {
            let room_name = pos.room_name();
            // TODO:
            // This lookup might be expensive, idk how to make it better right now.
            // Thats a problem for tomorrow.
            let new_path = new_roads.entry(room_name).or_insert_with(Vec::new);
            new_path.push(pos);

            let path_entry = paths.entry(room_name).or_insert_with(Vec::new);
            path_entry.push(pos);
        }
    }

    paths
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn room_callback(
    room_name: &RoomName,
    room_cache: &mut RoomCache,
    memory: &ScreepsMemory,
    new_roads: HashMap<RoomName, Vec<Position>>,
    existing_roads: HashMap<RoomName, Vec<Position>>,
) -> MultiRoomCostResult {
    let mut matrix = LocalCostMatrix::default();
    let terrain = game::map::get_room_terrain(*room_name)
        .unwrap()
        .get_raw_buffer()
        .to_vec();

    for x in 0..50 {
        for y in 0..50 {
            let tile = terrain[y * 50 + x];

            // FUCK pservers dude, like, what the hell.
            if tile == 1 || tile == 3 {
                matrix.set(unsafe { RoomXY::unchecked_new(x as u8, y as u8) }, 255);
                continue;
            }

            if tile & WALL_MASK != 0 {
                matrix.set(unsafe { RoomXY::unchecked_new(x as u8, y as u8) }, 255);
            } else if tile & SWAMP_MASK != 0 {
                matrix.set(unsafe { RoomXY::unchecked_new(x as u8, y as u8) }, 12);
            } else if tile == 0 {
                matrix.set(unsafe { RoomXY::unchecked_new(x as u8, y as u8) }, 10);
            } else {
                // Pserver wackiness
                // Impassible.
                matrix.set(unsafe { RoomXY::unchecked_new(x as u8, y as u8) }, 255);
            }
        }
    }

    if let Some(roads) = new_roads.get(room_name) {
        for road in roads {
            let xy = road.xy();
            matrix.set(xy, 1);
        }
    }

    if let Some(roads) = existing_roads.get(room_name) {
        for road in roads {
            let xy = road.xy();
            matrix.set(xy, 1);
        }
    }

    if let Some(room_cache) = room_cache.rooms.get_mut(room_name) {
        for road in room_cache.structures.roads.values() {
            let xy = road.pos().xy();
            matrix.set(xy, 1);
        }

        for structure in room_cache.structures.all_structures() {
            if structure.structure_type() == StructureType::Road {
                continue;
            }

            let walkable = WALKABLE_STRUCTURES.contains(&structure.structure_type());

            if walkable {
                let xy = structure.pos().xy();
                matrix.set(xy, 10);
            } else {
                let xy = structure.pos().xy();
                matrix.set(xy, 255);
            }
        }
    }

    if let Some(room_memory) = memory.rooms.get(room_name) {
        let offset_x = room_memory.spawn_center.x.u8();
        let offset_y = room_memory.spawn_center.y.u8() + 1;

        let all_plans = get_all_structure_plans();

        for plan in all_plans {
            if plan.2 == StructureType::Road {
                continue;
            }

            let x = plan.0 as u8 + offset_x;
            let y = plan.1 as u8 + offset_y;

            let xy = RoomXY::new(
                RoomCoordinate::new(x).unwrap(),
                RoomCoordinate::new(y).unwrap(),
            );

            if WALKABLE_STRUCTURES.contains(&plan.2) {
                matrix.set(xy, 10);
            } else {
                matrix.set(xy, 255);
            }
        }
    }

    MultiRoomCostResult::CostMatrix(matrix.into())
}
