use std::collections::HashMap;

use log::error;
use screeps::{ObjectId, Source, StructureController, StructureSpawn, Room, find, HasTypedId};
use serde::{Deserialize, Serialize};

use js_sys::JsString;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Careers {
    Mining,
    Odd
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Task {
    // Mining industry
    Miner(ObjectId<Source>),
    Hauler(ObjectId<StructureSpawn>),

    // Odd industry
    Rename(ObjectId<StructureController>)
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
    room_type: String,
    pub sources: HashMap<String, pub struct {
        pub id: String,
        pub spot_count: u8,
        pub spots: Vec<pub struct {
            pub x: u8,
            pub y: u8
        }>
    }>,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsMemory {
        pub creeps: HashMap<String, CreepMemory>,
        pub rooms: HashMap<String, RoomMemory>
}
}

impl ScreepsMemory {
    pub fn init_memory() -> Self {
        let memory_jsstring = screeps::raw_memory::get();
        let memory_string = memory_jsstring.as_string().unwrap();
        if memory_string.is_empty() {
            let memory = ScreepsMemory {
                creeps: HashMap::new(),
                rooms: HashMap::new(),
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
                        creeps: HashMap::new(),
                        rooms: HashMap::new(),
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

    pub fn create_creep(&mut self, name: &str, room: Room) {
        self.creeps.insert(
            name.to_string(),
            CreepMemory {
                movement: None,
                work: Some(Work {
                    career: Careers::Mining,
                    task: Some(Task::Miner(room.find(find::SOURCES, None).first().unwrap().id()))
                }),
            },
        );
    }

    pub fn create_room(&mut self, name: &str) {
        self.rooms.insert(
            name.to_string(),
            RoomMemory {
                room_type: "local".to_string(),
                sources: HashMap::new(),
            },
        );
    }
}
