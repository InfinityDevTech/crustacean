use std::collections::HashMap;

use log::{error, info};
use screeps::{ObjectId, Source, StructureController, Structure};
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

    // Odd industry
    Rename(ObjectId<StructureController>),
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
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct RoomMemory{
    // Name
    pub n: String,
    // Room type
    pub r_t: String,
    // Creeps
    pub cs: Vec<String>,
    // Creeps made
    pub c_m: u64,
    // Initialised
    pub init: bool,
    // Available mining spots, makes my life easier.
    pub avs: u8,
    // Mining stuffs
    pub mine: HashMap<ObjectId<Source>, pub struct {
        pub s: u8,
        pub u: u8,
    }>,
    // Creep Count
    pub c_c: pub struct {
        pub miner: u8,
        pub hauler: u8,
        pub upgrader: u8,
    }
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsMemory {
        pub rooms: HashMap<String, RoomMemory>,
        pub creeps: HashMap<String, CreepMemory>,
        pub stats: Option<pub struct {
            pub crm: u64,
        }>,
        pub spawn_tick: bool
}
}

impl ScreepsMemory {
    pub fn init_memory() -> Self {
        let memory_jsstring = screeps::raw_memory::get();
        let memory_string = memory_jsstring.as_string().unwrap();
        if memory_string.is_empty() {
            let memory = ScreepsMemory {
                rooms: HashMap::new(),
                creeps: HashMap::new(),
                stats: None,
                spawn_tick: true,
            };
            memory.write_memory();
            memory
        } else {
            match serde_json::from_str(&memory_string) {
                Ok(memory) => memory,
                Err(e) => {
                    error!("Error parsing memory: {}", e);
                    error!("This is a critical error, memory MUST be reset to default state.");
                    ScreepsMemory {
                        rooms: HashMap::new(),
                        creeps: HashMap::new(),
                        stats: None,
                        spawn_tick: true,
                    }
                }
            }
        }
    }
    pub fn write_memory(&self) {
        let serialized = serde_json::to_string(&self).unwrap();
        let js_serialized = JsString::from(serialized);
        screeps::raw_memory::set(&js_serialized);
    }

    pub fn create_creep(&mut self, room_name: &str, creep_name: &str, career: Careers, task: Option<Task>) {
        let room = self.rooms.get_mut(room_name).unwrap();
        let creep = CreepMemory {
            p: None,
            o_r: room_name.to_string(),
            c: career,
            t: task,
        };
        room.cs.push(creep_name.to_string());
        self.creeps.insert(creep_name.to_string(), creep);
        info!("Created creep");
    }

    pub fn create_room(&mut self, name: &str) {
        self.rooms.insert(
            name.to_string(),
            RoomMemory {
                n: name.to_string(),
                r_t: "local".to_string(),
                init: false,
                cs: Vec::new(),
                c_m: 0,
                avs: 0,
                mine: HashMap::new(),
                c_c: CC {
                    miner: 0,
                    hauler: 0,
                    upgrader: 0
                },
            },
        );
    }
}
