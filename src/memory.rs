use std::collections::HashMap;

use log::{error, info};
use screeps::{game, ObjectId, Source, Structure, StructureController};
use serde::{Deserialize, Serialize};

use js_sys::JsString;
use serde_json::Value;

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

    Attacker(),
    Healer(),
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct CreepMemory{
    // Owning room
    #[serde(default)]
    pub o_r: String,
    // Path
    #[serde(default)]
    pub p: Option<String>,
    // Career
    pub c: Careers,
    // Task
    #[serde(default)]
    pub t: Option<Task>,
    // State
    #[serde(default)]
    pub s: String,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct RoomMemory{
    // Name
    #[serde(default)]
    pub name: String,
    // Room type
    #[serde(default)]
    pub room_type: String,
    // Creeps made
    #[serde(default)]
    pub creeps_made: u64,
    // Initialised
    #[serde(default)]
    pub init: bool,
    // Available mining spots, makes my life easier.
    #[serde(default)]
    pub available_mining: u8,
    // Mining stuffs
    #[serde(default)]
    pub mine: HashMap<ObjectId<Source>, pub struct {
        #[serde(default)]
        pub s: u8,
        #[serde(default)]
        pub u: u8,
    }>,
    // Creeps by role
    #[serde(default)]
    pub creeps: HashMap<String, String>
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone, Default)]]
    pub struct ScreepsMemory {
        #[serde(default)]
        pub rooms: HashMap<String, RoomMemory>,
        #[serde(default)]
        pub creeps: HashMap<String, CreepMemory>,
        #[serde(default)]
        pub stats: pub struct {
            #[serde(default)]
            pub cpu: pub struct {
                #[serde(default)]
                pub memory: f64,
                #[serde(default)]
                pub rooms: f64,
                #[serde(default)]
                pub total: f64,
                #[serde(default)]
                pub bucket: i32,
            },
            #[serde(default)]
            pub rooms: HashMap<String, pub struct {
                #[serde(default)]
                pub cpu: f64,
                #[serde(default)]
                pub mining: f64,
                #[serde(default)]
                pub construction: f64,
                #[serde(default)]
                pub rcl: u8,
                #[serde(default)]
                pub creeps_made: u64,
                #[serde(default)]
                pub creeps_removed: u64,
                #[serde(default)]
                pub energy_harvested: u64,
                #[serde(default)]
                pub energy_harvested_total: u64,
                #[serde(default)]
                pub energy_available: u64,
                #[serde(default)]
                pub energy_capacity_available: u64
            }>,
            pub energy_harvested: u64,
        },
        #[serde(default)]
        pub spawn_tick: bool,
}
}

unsafe impl Send for ScreepsMemory {}
unsafe impl Sync for ScreepsMemory {}

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

    //pub fn default()
}