use screeps::{CostMatrix, CostMatrixSet, HasPosition, LocalCostMatrix, OwnedStructureProperties, Room, Structure, StructureObject, StructureProperties, StructureType};

use crate::room::cache::RoomCache;

pub fn owned_room(room: &Room, cache: &mut RoomCache) -> LocalCostMatrix {
    let mut matrix = LocalCostMatrix::new();

    for structure in &cache.structures.all_structures {

        let score = match structure {
            StructureObject::StructureRoad(_) => 1,
            StructureObject::StructureRampart(rampart) => {
                if rampart.my() {
                    2
                } else {
                    255
                }
            },
            StructureObject::StructureContainer(_) => 2,
            _ => 255,
        };

        matrix.set(structure.pos().xy(), score);
    }

    for construction_site in &cache.structures.construction_sites {
        matrix.set_xy(construction_site.pos().xy(), 2);
    }

    for road in cache.structures.roads.values() {
        matrix.set_xy(road.pos().xy(), 1);
    }

    matrix
}