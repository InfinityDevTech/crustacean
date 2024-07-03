use screeps::{game, pathfinder::{self, MultiRoomCostResult, PathFinder, SearchOptions}, CostMatrix, HasPosition, LocalCostMatrix, Position, Room, RoomCoordinate, RoomName};

use crate::{memory::ScreepsMemory, room::cache::tick_cache::CachedRoom};

pub fn plan_main_room_roads(room: &Room, cache: &CachedRoom, memory: &mut ScreepsMemory) {
    for source in &cache.resources.sources {
        let origin_position = if cache.structures.storage.is_some() {
            cache.structures.storage.as_ref().unwrap().pos()
        } else {
            let pos = cache.spawn_center;

            let y = pos.y.u8();

            let y = RoomCoordinate::new(y - 2).unwrap();

            Position::new(pos.x, y, room.name())
        };

        let game_source = game::get_object_by_id_typed(&source.id).unwrap();
        let destination_position = game_source.pos();

        let pathfinder_options = SearchOptions::new(|room_name| room_callback(&room_name, cache)).plain_cost(2).swamp_cost(2);
        let result = pathfinder::search(origin_position, destination_position, 1, Some(pathfinder_options));

        if result.incomplete() {
            log::error!("Pathfinding failed for source {:?}", source.id);
            continue;
        }

        let path = result.path();

        for pos in path {
            let x = pos.x().u8();
            let y = pos.y().u8();

            let _ = room.create_construction_site(x, y, screeps::StructureType::Road, None);
        }
    }
}

fn room_callback(room_name: &RoomName, room_cache: &CachedRoom) -> MultiRoomCostResult {
    let mut local_matrix = LocalCostMatrix::default();

    for road in room_cache.structures.roads.values() {
        let xy = road.pos().xy();
        local_matrix.set(xy, 1);
    }

    MultiRoomCostResult::CostMatrix(local_matrix.into())
}