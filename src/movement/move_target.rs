use log::{info, warn};
use screeps::{
    find, game,
    pathfinder::{self, MultiRoomCostResult, SearchOptions},
    HasPosition, LocalCostMatrix, OwnedStructureProperties, Position, RoomName, Structure,
    StructureObject, StructureType,
};

use crate::cache::ScreepsCache;

pub struct MoveTarget {
    pub pos: Position,
    pub range: u32,
}

impl MoveTarget {
    pub fn find_path_to(&mut self, from: Position, cache: &mut ScreepsCache) -> String {
        let opts = SearchOptions::new(|room_name: RoomName| path_call(room_name, cache))
            .plain_cost(2)
            .swamp_cost(5)
            .max_rooms(20)
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

pub fn path_call(room_name: RoomName, cache: &mut ScreepsCache) -> MultiRoomCostResult {
    if cache.cost_matrixes.get(&room_name.to_string()).is_none() {
        let starting_cpu = game::cpu::get_used();
        let mut matrix = LocalCostMatrix::new();
        if let Some(room) = screeps::game::rooms().get(room_name) {
            info!("     Room get CPU {}", game::cpu::get_used() - starting_cpu);
            let structures = cache.structures.values();
            info!(
                "     Structures get CPU {}",
                game::cpu::get_used() - starting_cpu
            );
            if let Some(constructions) = cache.csites.get(&room_name.to_string()) {
                for csite_id in constructions {
                    let csite = csite_id.resolve().unwrap();
                    if csite.room().unwrap().name() != room_name {
                        continue;
                    }
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
            }
            info!(
                "     Constructions get CPU {}",
                game::cpu::get_used() - starting_cpu
            );
            let creeps = room.find(find::CREEPS, None);
            info!("     Structure count {}", structures.len());
            for structure_id in structures.flatten() {
                let structure = structure_id.resolve().unwrap();
                if structure.room().unwrap().name() != room_name {
                    continue;
                }
                let pos = structure.pos();
                match StructureObject::from(structure) {
                    StructureObject::StructureContainer(_) => matrix.set(pos.xy(), 1),
                    StructureObject::StructureRampart(rampart) => {
                        if rampart.my() {
                            matrix.set(pos.xy(), 1);
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
            info!(
                "     Structures write CPU {}",
                game::cpu::get_used() - starting_cpu
            );

            info!(
                "     Constructions CPU {}",
                game::cpu::get_used() - starting_cpu
            );

            for creep in creeps {
                let pos = creep.pos();
                matrix.set(pos.xy(), 255);
            }

            info!("     Creeps CPU {}", game::cpu::get_used() - starting_cpu);
        }
        info!("     Total CPU {}", game::cpu::get_used() - starting_cpu);
        cache
            .cost_matrixes
            .insert(room_name.to_string(), matrix.clone());
        info!("Cached matrix!");
        MultiRoomCostResult::CostMatrix(matrix.into())
    } else {
        info!("Returned cached matrix!");
        MultiRoomCostResult::CostMatrix(
            cache
                .cost_matrixes
                .get(&room_name.to_string())
                .unwrap()
                .clone()
                .into(),
        )
    }
}
