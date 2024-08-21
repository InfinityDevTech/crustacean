use screeps::{game, HasPosition, LocalCostMatrix, Room, RoomCoordinate, RoomXY, StructureProperties, Terrain};

use crate::{constants::WALKABLE_STRUCTURES, heap, compression::compressed_matrix::CompressedMatrix, memory::ScreepsMemory, room::cache::CachedRoom, traits::position::PositionExtensions};

use super::flow_field::FlowField;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn generate_pathing_targets(room: &Room, _memory: &ScreepsMemory, room_cache: &mut CachedRoom) {
    let mut room_target_heap = heap().cachable_positions.lock().unwrap();

    let mut positions = Vec::new();

    // Controller and its containers.
    if room_cache.structures.controller.is_some() {
        if room_cache.structures.containers().controller.is_some() {
            positions.push(room_cache.structures.containers().controller.as_ref().unwrap().pos());
        }

        positions.push(room_cache.structures.controller.as_ref().unwrap().pos());
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

    if let Some(storage) = &room_cache.structures.storage {
        positions.push(storage.pos());
    }

    if let Some(fast_filler_containers) = &room_cache.structures.containers().fast_filler {
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
pub fn generate_storage_path(room: &Room, room_cache: &mut CachedRoom) -> CompressedMatrix {
    let mut flow_field = FlowField::new(50, 50, true);

    let all_structures = room_cache.structures.all_structures().clone();
    let construction_sites = room_cache.structures.construction_sites.clone();

    let callback = || {
        let mut matrix = LocalCostMatrix::new();
        let terrain = game::map::get_room_terrain(room.name()).unwrap();

        for x in 0..50 {
            for y in 0..50 {
                let pos = RoomXY::new(RoomCoordinate::new(x).unwrap(), RoomCoordinate::new(y).unwrap());
                let terrain = terrain.get(x, y);

                match terrain {
                    Terrain::Plain => matrix.set(pos, 3),
                    Terrain::Swamp => matrix.set(pos, 5),
                    Terrain::Wall => matrix.set(pos, 255),
                }

                // Add a border around the room.
                if x == 0 || y == 0 || x == 49 || y == 49 {
                    matrix.set(pos, 255);
                }
            }
        }

        for rampart in &room_cache.structures.ramparts {
            matrix.set(rampart.pos().xy(), 1);
        }

        for road in room_cache.structures.roads.values() {
            matrix.set(road.pos().xy(), 1);
        }

        for structure in &all_structures {
            if !WALKABLE_STRUCTURES.contains(&structure.structure_type()) {
                matrix.set(structure.pos().xy(), 255);
            }
        }

        for construction_site in &construction_sites {
            if !WALKABLE_STRUCTURES.contains(&construction_site.structure_type()) {
                matrix.set(construction_site.pos().xy(), 255);
            }
        }

        // Storage sitter.
        if let Some(storage_pos) = room_cache.storage_center {
            matrix.set(storage_pos, 255);
        }

        // Fast fillers
        if let Some(spawn_center) = room_cache.spawn_center {
            let y = RoomCoordinate::new(spawn_center.y.u8()).unwrap();

            let xy1 = RoomXY::new(RoomCoordinate::new(spawn_center.x.u8() + 1).unwrap(), y);
            let xy2 = RoomXY::new(RoomCoordinate::new(spawn_center.x.u8() - 1).unwrap(), y);
            matrix.set(xy1, 255);
            matrix.set(xy2, 255);
        }

        matrix
    };

    //let cache = heap().flow_cache.lock().unwrap().get_mut(&room.name()).unwrap();
    //*cache.storage = Some(field.clone());

    flow_field.generate(vec![room_cache.structures.storage.as_ref().unwrap().pos().xy()], callback, None)
}