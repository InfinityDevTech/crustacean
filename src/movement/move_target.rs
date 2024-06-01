use log::warn;
use screeps::{
    pathfinder::{self, MultiRoomCostResult, SearchOptions}, HasPosition, LocalCostMatrix, OwnedStructureProperties, Position,
    RoomName, StructureObject, find, StructureType
};

pub struct MoveTarget {
    pub pos: Position,
    pub range: u32
}

impl MoveTarget {
    pub fn find_path_to(&mut self, from: Position) -> String {
        let opts = SearchOptions::new(path_call)
            .plain_cost(2)
            .swamp_cost(5)
            .max_rooms(1)
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
        let steps = &steps[0..std::cmp::min(steps.len(), 10)];
        for dirint in steps {
            let int = *dirint as u8;
            let intstring = int.to_string();

            steps_string = steps_string + &intstring;
        }
        steps_string
    }
}

pub fn path_call(room_name: RoomName) -> MultiRoomCostResult {
    let mut matrix = LocalCostMatrix::new();
    if let Some(room) = screeps::game::rooms().get(room_name) {
        let structures = room.find(find::STRUCTURES, None);
        let constructions = room.find(find::CONSTRUCTION_SITES, None);
        let creeps = room.find(find::CREEPS, None);

        for csite in constructions {
            let pos = csite.pos();
            match csite.structure_type() {
                StructureType::Container => matrix.set(pos.xy(), 2),
                StructureType::Rampart => matrix.set(pos.xy(), 2),
                StructureType::Road => matrix.set(pos.xy(), 2),
                StructureType::Wall => matrix.set(pos.xy(), 255),
                _ => {
                    matrix.set(pos.xy(), 255);
                }
            }
        }

        for structure in structures {
            let pos = structure.pos();
            match structure {
                StructureObject::StructureContainer(_) => matrix.set(pos.xy(), 2),
                StructureObject::StructureRampart(rampart) => {
                    if rampart.my() {
                        matrix.set(pos.xy(), 2);
                    } else {
                        matrix.set(pos.xy(), 255);
                    }
                }
                StructureObject::StructureRoad(_) => matrix.set(pos.xy(), 1),
                StructureObject::StructureWall(_) => matrix.set(pos.xy(), 255),
                _ => {
                    matrix.set(pos.xy(), 255);
                }
            }
        }

        //for creep in creeps {
        //    let pos = creep.pos();
        //    matrix.set(pos.xy(), 255);
        //}
    }
    MultiRoomCostResult::CostMatrix(matrix.into())
}
