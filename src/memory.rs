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
    pub movement: Option<pub struct {
        pub dest: struct {
        pub x: u8,
        pub y: u8,
        pub room: String
    },
    pub path: String,
    pub room: String
    }>,
    pub work: Option<pub struct {
        pub career: Careers,
        pub task: Option<Task>,
    }>
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct RoomMemory{
    pub name: String,
    pub room_type: String,
    pub creeps: HashMap<String, CreepMemory>,
    pub sources: HashMap<String, pub struct {
        pub id: String,
        pub spot_count: u8,
        pub spots: Vec<pub struct {
            pub x: u8,
            pub y: u8
        }>
    }>,
    pub creep_count: pub struct {
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
                        spawn_tick: true
                    }
                }
            }
        }
    }
    pub fn write_memory(&self) {
        info!("Writing memory");
        let serialized = serde_json::to_string(&self).unwrap();
        let js_serialized = JsString::from(serialized);
        screeps::raw_memory::set(&js_serialized);
    }

    pub fn create_creep(&mut self, room_name: &str, creep_name: &str, task: Task) {
        self.rooms.get_mut(room_name).unwrap().creeps.insert(
            creep_name.to_string(),
            CreepMemory {
                movement: None,
                work: Some(Work {
                    career: Careers::Mining,
                    task: Some(task),
                }),
            },
        );
        info!("Created creep");
    }

    pub fn create_room(&mut self, name: &str) {
        self.rooms.insert(
            name.to_string(),
            RoomMemory {
                name: name.to_string(),
                room_type: "local".to_string(),
                creeps: HashMap::new(),
                sources: HashMap::new(),
                creep_count: CreepCount {
                    miner: 0,
                    hauler: 0,
                    upgrader: 0
                },
            },
        );
    }
}
