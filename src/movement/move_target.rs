use log::warn;
use screeps::{
    find, pathfinder::{self, MultiRoomCostResult, SearchOptions, SearchResults}, HasPosition, LocalCostMatrix, LocalRoomTerrain, OwnedStructureProperties, Part, Position, RoomName, RoomXY, StructureObject, StructureType
};

#[derive(Debug, Clone, Copy)]
pub struct MoveOptions {
    pub avoid_enemies: bool,
    pub path_age: u8,
}

impl Default for MoveOptions {
    fn default() -> Self {
        MoveOptions {
            avoid_enemies: false,
            path_age: 8,
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl MoveOptions {
    pub fn avoid_enemies(&mut self, avoid_enemies: bool) -> Self {
        self.avoid_enemies = avoid_enemies;
        *self
    }

    pub fn path_age(&mut self, path_age: u8) -> Self {
        self.path_age = path_age;
        *self
    }
}

pub struct MoveTarget {
    pub pos: Position,
    pub range: u32
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl MoveTarget {
    pub fn find_path_to(&mut self, from: Position, move_options: MoveOptions) -> String {
        //info!("Finding path to {}", self.pos);
        let opts = SearchOptions::new(|room_name| {
            path_call(room_name, move_options)
        })
            .max_rooms(15)
            .max_ops(20000);

        let search = self.pathfind(from, Some(opts));
        

        self.serialize_path(from, search.clone().into(), move_options)
    }

    pub fn pathfind(&mut self, from: Position, opts: Option<SearchOptions<impl FnMut(RoomName) -> MultiRoomCostResult>>) -> SearchResults {
        pathfinder::search(from, self.pos, self.range, opts)
    }

    pub fn serialize_path(&mut self, from: Position, search: SearchResults, move_options: MoveOptions) -> String {
        let mut cur_pos = from;
        let positions = search.path();
        let mut steps = Vec::with_capacity(positions.len());

        let path_age = move_options.path_age as usize;

        for pos in positions {
            if steps.len() >= path_age {
                break;
            }

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
        let steps = &steps[0..std::cmp::min(steps.len(), move_options.path_age as usize)];

        for dirint in steps {
            let int = *dirint as u8;
            let intstring = int.to_string();

            steps_string = steps_string + &intstring;
        }

        steps_string
    }
}

//pub const TEMP_COUNT: Mutex<u8> = Mutex::new(0);

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn path_call(room_name: RoomName, move_options: MoveOptions) -> MultiRoomCostResult {
    let mut matrix = LocalCostMatrix::new();

    if let Some(room) = screeps::game::rooms().get(room_name) {
        let structures = room.find(find::STRUCTURES, None);
        let constructions = room.find(find::CONSTRUCTION_SITES, None);
        let creeps = room.find(find::CREEPS, None);
        let terrain = LocalRoomTerrain::from(room.get_terrain());

        for x in 0..50 {
            for y in 0..50 {
                let pos = unsafe { RoomXY::unchecked_new(x, y) };
                let tile = terrain.get_xy(pos);

                match tile {
                    screeps::Terrain::Plain => matrix.set(pos, 1),
                    screeps::Terrain::Wall => matrix.set(pos, 255),
                    screeps::Terrain::Swamp => matrix.set(pos, 5),
                }
            }
        }

        for csite in constructions {
            let pos = csite.pos();

            if !csite.my() {
                matrix.set(pos.xy(), 2);
                continue;
            }

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
                    if !rampart.my() {
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

        for creep in creeps {
            let pos = creep.pos();
            matrix.set(pos.xy(), 3);
        }

        if move_options.avoid_enemies {
            let enemies = room.find(find::HOSTILE_CREEPS, None);
            for enemy in enemies {
                if enemy.body().iter().filter(|p| p.part() == Part::Attack || p.part() == Part::RangedAttack && p.hits() > 0).count() == 0 {
                    continue;
                }

                let radius = 3;

                let start_x = enemy.pos().x().u8();
                let start_y = enemy.pos().y().u8();

                for x in start_x - radius..=start_x + radius {
                    for y in start_y - radius..=start_y + radius {
                        if x == start_x && y == start_y {
                            continue;
                        }

                        let xy = unsafe { RoomXY::unchecked_new(x, y) };

                        matrix.set(xy, 255);
                    }
                }
            }
        }
    }

    /*let t = TEMP_COUNT;
    let mut count = t.lock().unwrap();

    if let Some(vis) = game::rooms().get(room_name) {
        if room_name != "W3N12" && *count < 1 {
            return MultiRoomCostResult::CostMatrix(matrix.into());
        }

        let vis = vis.visual();

        for x in 0..50 {
            for y in 0..50 {
                let score = matrix.get(unsafe { RoomXY::unchecked_new(x, y) });

                let color = if score == 2 {
                    "green"
                } else if score == 1 {
                    "white"
                } else if score == 5 {
                    "blue"
                } else if score == 255 {
                    "red"
                } else {
                    "black"
                };

                let style = RectStyle::default().fill(color).opacity(0.2);
                vis.rect(x as f32 - 0.5, y as f32 - 0.5, 1.0, 1.0, Some(style));
                //vis.text(x as f32 - 0.5, y as f32 - 0.5, format!("{}", score), Some(Default::default()));
            }
        }
    }

    *count += 1;*/
    MultiRoomCostResult::CostMatrix(matrix.into())
}
