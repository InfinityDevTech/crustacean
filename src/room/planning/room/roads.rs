use screeps::{pathfinder::{self, MultiRoomCostResult, SearchOptions}, HasPosition, LocalCostMatrix, Position, Room, RoomCoordinate, RoomName, StructureProperties};

use crate::{memory::ScreepsMemory, room::cache::CachedRoom};

use super::convert_path_to_roads;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn plan_main_room_roads(room: &Room, cache: &mut CachedRoom, _memory: &mut ScreepsMemory) {
    for source in &cache.resources.sources.clone() {
        let origin_position = if cache.structures.storage.is_some() {
            cache.structures.storage.as_ref().unwrap().pos()
        } else {
            let pos = cache.spawn_center.unwrap();

            let y = pos.y.u8();

            let y = RoomCoordinate::new(y - 2).unwrap();

            Position::new(pos.x, y, room.name())
        };

        let game_source = &source.source;
        let destination_position = game_source.pos();

        let pathfinder_options = SearchOptions::new(|room_name| room_callback(&room_name, cache)).plain_cost(2).swamp_cost(2);
        let result = pathfinder::search(origin_position, destination_position, 1, Some(pathfinder_options));

        convert_path_to_roads(room, result);
    }

    if let Some(mineral) = &cache.resources.mineral {
        let origin_position = if cache.structures.storage.is_some() {
            cache.structures.storage.as_ref().unwrap().pos()
        } else {
            let pos = cache.spawn_center.unwrap();

            let y = pos.y.u8();

            let y = RoomCoordinate::new(y - 2).unwrap();

            Position::new(pos.x, y, room.name())
        };

        let destination_position = mineral.pos();

        let pathfinder_options = SearchOptions::new(|room_name| room_callback(&room_name, cache)).plain_cost(2).swamp_cost(2);
        let result = pathfinder::search(origin_position, destination_position, 1, Some(pathfinder_options));

        convert_path_to_roads(room, result);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn room_callback(_room_name: &RoomName, room_cache: &mut CachedRoom) -> MultiRoomCostResult {
    let mut local_matrix = LocalCostMatrix::default();

    for road in room_cache.structures.roads.values() {
        let xy = road.pos().xy();
        local_matrix.set(xy, 1);
    }

    for structure in &room_cache.structures.all_structures() {
        let walkable = matches!(structure.structure_type(), screeps::StructureType::Road | screeps::StructureType::Container | screeps::StructureType::Rampart);

        if walkable {
            let xy = structure.pos().xy();
            local_matrix.set(xy, 1);
        } else {
            let xy = structure.pos().xy();
            local_matrix.set(xy, 255);
        }
    }

    MultiRoomCostResult::CostMatrix(local_matrix.into())
}