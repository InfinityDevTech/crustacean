use log::info;
use screeps::{game, CostMatrixGet, HasPosition, LocalCostMatrix, Room, RoomCoordinate, RoomXY, StructureProperties, StructureType, Terrain, TextStyle};

use crate::{constants::WALKABLE_STRUCTURES, heap, heap_cache::CompressedDirectionMatrix, memory::ScreepsMemory, movement::flow_field::visualise_field, room::cache::{self, tick_cache::CachedRoom}, traits::position::PositionExtensions};

use super::flow_field::{FlowField, FlowFieldSource};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn generate_pathing_targets(room: &Room, memory: &ScreepsMemory, room_cache: &mut CachedRoom) {
    let mut room_target_heap = heap().cachable_positions.lock().unwrap();

    let mut positions = Vec::new();

    // Controller and its containers.
    if let Some(controller) = &room_cache.structures.controller {
        if controller.container.is_some() {
            positions.push(controller.container.as_ref().unwrap().pos());
        }

        positions.push(controller.controller.pos());
    }

    // Source and its containers.
    // TODO: Actually cache source.
    for source in &room_cache.resources.sources {
        if source.container.is_some() {
            positions.push(source.container.as_ref().unwrap().pos());
        } else {
            let ppositions = source.source.pos().get_accessible_positions_around(1);

            for pos in ppositions {
                positions.push(pos);
            }
        }

        positions.push(source.source.pos());
    }

    if let Some(fast_filler_containers) = &room_cache.structures.containers.fast_filler {
        for container in fast_filler_containers {
            positions.push(container.pos());
        }
    }

    if let Some(theap) = room_target_heap.get_mut(&room.name()) {
        theap.clear();
        theap.extend(positions);
    } else {
        room_target_heap.insert(room.name(), positions);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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