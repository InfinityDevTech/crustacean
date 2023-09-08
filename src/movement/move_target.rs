use log::warn;
use screeps::{
    find, game,
    pathfinder::{self, MultiRoomCostResult, SearchOptions},
    HasPosition, LocalCostMatrix, OwnedStructureProperties, Position, RoomName, StructureObject, StructureType,
};

use crate::{cache::ScreepsCache, traits::room::RoomExtensions};

pub struct MoveTarget {
    pub pos: Position,
    pub range: u32,
}

impl MoveTarget {
    pub fn find_path_to(&mut self, from: Position, cache: &mut ScreepsCache) -> String {
        let opts = SearchOptions::new(|room_name: RoomName| path_call(room_name, cache))
            .plain_cost(2)
            .swamp_cost(5)
            .max_rooms(10)
            .max_ops(100000);
        let search = pathfinder::search(from, self.pos, self.range, Some(opts));

        if search.incomplete() {
            //warn!(
            //    "Incomplete pathfinding search {} {} {}",
            //    search.ops(),
            //    search.cost(),
            //    self.pos
            //);
        }

        let mut cur_pos = from;
        let positions = search.path();
        let mut steps = Vec::with_capacity(positions.len());
        for pos in positions {
            if pos.room_name() == cur_pos.room_name() {
                match pos.get_direction_to(cur_pos) {
                    Some(dir) => {
                        steps.push(-dir);
                    }
                    None => {
                        warn!("Couldn't get direction to {:?} from {:?}", pos, cur_pos);
                        break;
                    }
                }
            }
            cur_pos = pos;
        }
        let mut steps_string = "".to_string();
        let steps = &steps[0..std::cmp::min(steps.len(), 5)];
        for dirint in steps {
            let int = *dirint as u8;
            let intstring = int.to_string();

            steps_string = steps_string + &intstring;
        }
        steps_string
    }
}

pub fn matrix_generation(room_name: RoomName, cache: &mut ScreepsCache) -> MultiRoomCostResult {
    let mut matrix = LocalCostMatrix::new();

        let structures = cache.get_room(&room_name.to_string()).unwrap().structures.values().flatten();
        for structure in structures {
            let structure = structure.resolve().unwrap();
            let pos = structure.pos();
            match StructureObject::from(structure) {
                StructureObject::StructureContainer(_) => matrix.set(pos.xy(), 2),
                StructureObject::StructureRampart(rampart) => if rampart.my() {matrix.set(rampart.pos().xy(), 1)} else {matrix.set(rampart.pos().xy(), 255)},
                StructureObject::StructureRoad(_) => matrix.set(pos.xy(), 1),
                StructureObject::StructureWall(_) => matrix.set(pos.xy(), 255),
                _ => matrix.set(pos.xy(), 255)
            }
        }

        let csites = cache.get_room(&room_name.to_string()).unwrap().csites.clone();
        for csite in csites {
            let csite = csite.resolve().unwrap();
            match csite.structure_type() {
                StructureType::Road => matrix.set(csite.pos().xy(), 2),
                StructureType::Wall => matrix.set(csite.pos().xy(), 255),
                StructureType::Container => matrix.set(csite.pos().xy(), 2),
                _ => matrix.set(csite.pos().xy(), 255),
            }
        }

        let creeps = game::rooms().get(room_name).unwrap().find(find::CREEPS, None);
        for creep in creeps {
            if creep.my() {
                matrix.set(creep.pos().xy(), 255);
            }
        }

        cache.get_room(&room_name.to_string()).unwrap().cost_matrix = Some(matrix.clone());

        MultiRoomCostResult::CostMatrix(matrix.into())
}

pub fn path_call(room_name: RoomName, cache: &mut ScreepsCache) -> MultiRoomCostResult {
    if cache.get_room(&room_name.to_string()).is_none() {
        if let Some(room) = game::rooms().get(room_name) {
            room.initialise_cache(cache);
        } else {
            return MultiRoomCostResult::default();
        }

        matrix_generation(room_name, cache)
    } else if cache.get_room(&room_name.to_string()).unwrap().cost_matrix.is_none() {
        matrix_generation(room_name, cache)
    } else {
        MultiRoomCostResult::CostMatrix(cache.get_room(&room_name.to_string()).unwrap().cost_matrix.clone().unwrap().into())
    }
}
