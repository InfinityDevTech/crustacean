use screeps::{game, HasPosition, LocalCostMatrix, Room, RoomCoordinate, RoomXY, StructureProperties, StructureType, Terrain};

use crate::{constants::WALKABLE_STRUCTURES, heap, heap_cache::CompressedDirectionMatrix, room::cache::{self, tick_cache::CachedRoom}};

use super::flow_field::{FlowField, FlowFieldSource};

pub fn generate_storage_path(room: &Room, room_cache: &mut CachedRoom) -> CompressedDirectionMatrix {
    let mut flow_field = FlowField::new(50, 50, true);

    let callback = || {
        let mut matrix = LocalCostMatrix::new();
        let mut terrain = game::map::get_room_terrain(room.name()).unwrap();

        for x in 0..50 {
            for y in 0..50 {
                let pos = RoomXY::new(RoomCoordinate::new(x).unwrap(), RoomCoordinate::new(y).unwrap());
                let terrain = terrain.get(x, y);

                match terrain {
                    Terrain::Plain => matrix.set(pos, 3),
                    Terrain::Swamp => matrix.set(pos, 5),
                    Terrain::Wall => matrix.set(pos, 255),
                }
            }
        }


        for rampart in &room_cache.structures.ramparts {
            matrix.set(rampart.pos().xy(), 1);
        }

        for (road_id, road) in &room_cache.structures.roads {
            matrix.set(road.pos().xy(), 1);
        }

        for structure in &room_cache.structures.all_structures {
            if !WALKABLE_STRUCTURES.contains(&structure.structure_type()) {
                matrix.set(structure.pos().xy(), 255);
            }
        }

        matrix
    };

    let source = FlowFieldSource {
        pos: room_cache.structures.storage.as_ref().unwrap().pos().xy(),
        cost: 0,
    };

    let field = flow_field.generate(vec![source], callback, None);

    //let cache = heap().flow_cache.lock().unwrap().get_mut(&room.name()).unwrap();
    //*cache.storage = Some(field.clone());

    field
}