use std::collections::HashMap;

use log::{error, info};
use screeps::{game, ObjectId, Source, Structure, StructureController};
use serde::{Deserialize, Serialize};

use js_sys::JsString;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Careers {
    Mining,
    Odd,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Task {
    // Mining industry
    Miner(ObjectId<Source>),
    Hauler(ObjectId<Structure>),

    // Construction industry
    Upgrader(ObjectId<StructureController>),
    Builder(),

    // Odd industry
    Rename(ObjectId<StructureController>),

    Scout(),
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct CreepMemory{
    // Owning room
    pub o_r: String,
    // Path
    pub p: Option<String>,
    // Career
    pub c: Careers,
    // Task
    pub t: Option<Task>,
    // State
    pub s: String,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct RoomMemory{
    // Name
    pub name: String,
    // Room type
    pub room_type: String,
    // Creeps made
    pub creeps_made: u64,
    // Initialised
    pub init: bool,
    // Available mining spots, makes my life easier.
    pub available_mining: u8,
    // Mining stuffs
    pub mine: HashMap<ObjectId<Source>, pub struct {
        pub s: u8,
        pub u: u8,
    }>,
    // Creeps by role
    pub creeps: HashMap<String, String>
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsMemory {
        pub rooms: HashMap<String, RoomMemory>,
        pub creeps: HashMap<String, CreepMemory>,
        pub stats: pub struct {
            pub cpu: pub struct {
                pub memory: f64,
                pub rooms: f64,
                pub total: f64,
                pub bucket: i32,
            },
            pub rooms: HashMap<String, pub struct {
                pub cpu: f64,
                pub mining: f64,
                pub construction: f64,
                pub rcl: u8,
                pub creeps_made: u64,
                pub creeps_removed: u64,
                pub energy_harvested: u64,
                pub energy_harvested_total: u64,
                pub energy_available: u64,
                pub energy_capacity_available: u64
            }>,
            pub energy_harvested: u64,
        },
        pub spawn_tick: bool
}
}

impl ScreepsMemory {
    pub fn init_memory() -> Self {
        let memory_jsstring = screeps::raw_memory::get();
        let memory_string = memory_jsstring.as_string().unwrap();
        if memory_string.is_empty() {
            let mut memory = ScreepsMemory {
                rooms: HashMap::new(),
                creeps: HashMap::new(),
                stats: Stats { cpu: Cpu { memory: 0.0, rooms: 0.0, total: 0.0, bucket: 0 }, rooms: HashMap::new(), energy_harvested: 0 },
                spawn_tick: true,
            };
            memory.write_memory();
            memory
        } else {
            match serde_json::from_str::<ScreepsMemory>(&memory_string) {
                Ok(memory) => {
                    memory
                },
                Err(e) => {
                    error!("Error parsing memory: {}", e);
                    error!("This is a critical error, memory MUST be reset to default state.");
                    
                    ScreepsMemory {
                        rooms: HashMap::new(),
                        creeps: HashMap::new(),
                        stats: Stats { cpu: Cpu { memory: 0.0, rooms: 0.0, total: 0.0, bucket: 0 }, rooms: HashMap::new(), energy_harvested: 0 },
                        spawn_tick: true,
                    }
                }
            }
        }
    }

    pub fn write_memory(&mut self) {
        let starting_cpu = game::cpu::get_used();
        self.stats.cpu.memory += game::cpu::get_used() - starting_cpu;
        let serialized = serde_json::to_string(&self).unwrap();
        let js_serialized = JsString::from(serialized);
        screeps::raw_memory::set(&js_serialized);
        self.stats.cpu.memory += game::cpu::get_used() - starting_cpu;
    }

    pub fn create_creep(&mut self, room_name: &str, creep_name: &str, career: Careers, task: Option<Task>) {
        let creep = CreepMemory {
            p: None,
            o_r: room_name.to_string(),
            c: career,
            t: task,
            s: "energy".to_string(),
        };
        self.creeps.insert(creep_name.to_string(), creep);
        info!("Created creep");
    }

    pub fn create_room(&mut self, name: &str) {
        self.rooms.insert(
            name.to_string(),
            RoomMemory {
                name: name.to_string(),
                room_type: "local".to_string(),
                init: false,
                creeps_made: 0,
                available_mining: 0,
                mine: HashMap::new(),
                creeps: HashMap::new(),
            },
        );
    }

    pub fn get_room(&mut self, name: &str) -> &mut RoomMemory {
        self.rooms.get_mut(&name.to_string()).expect("Failed to get room from memory, attempted room name")
    }

    pub fn get_creep(&mut self, name: &str) -> &mut CreepMemory {
        self.creeps.get_mut(&name.to_string()).unwrap()
    }
}

impl RoomMemory {
    pub fn get_creeps_by_role(&self, role: &str) -> Vec<String> {
        self.creeps.clone().into_iter().filter(|x| x.1 == *role).map(|x| x.0).collect()
    }
}

impl Stats {
    pub fn create_room(&mut self, name: &str, rcl: u8) {
        self.rooms.insert(
            name.to_string(),
              Rooms {
                creeps_made: 0,
                mining: 0.0,
                construction: 0.0,
                rcl,
                creeps_removed: 0,
                cpu: 0.0,
                energy_harvested: 0,
                energy_harvested_total: 0,
                energy_available: 0,
                energy_capacity_available: 0
              }
        );
    }

    pub fn get_room(&mut self, name: &str) -> &mut Rooms {
        self.rooms.get_mut(&name.to_string()).expect("Failed to get room from stats")
    }
}