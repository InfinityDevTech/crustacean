use std::collections::HashMap;

use log::{error, info};
use screeps::{game, ObjectId, RoomName, Source, Structure, StructureController};
use serde::{Deserialize, Serialize};

use js_sys::JsString;

use crate::MEMORY_VERSION;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Role {
    // Mining industry
    Miner,
    Hauler,

    // Construction industry
    Upgrader,
    Builder,

    Scout,
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct CreepMemory{
    // Owning room
    pub o_r: String,
    // Path
    pub p: Option<String>,
    // Career
    pub r: Role,
    // Needs Energy?
    pub n_e: Option<bool>,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct RoomMemory{
    // Name
    pub name: String,
    // Initialised
    pub init: bool,
    // Mining stuffs
    pub sources: HashMap<ObjectId<Source>, pub struct {
        pub mining_spots: u8,
        pub assigned_creeps: u8,
    }>,
    // Creeps by role
    pub creeps: Vec<String>,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsMemory {
        pub mem_version: u8,
        pub rooms: HashMap<String, RoomMemory>,
        pub creeps: HashMap<String, CreepMemory>,
        pub spawn_tick: bool
    }
}

impl ScreepsMemory {
    pub fn init_memory() -> Self {
        let memory_jsstring = screeps::raw_memory::get();
        let memory_string = memory_jsstring.as_string().unwrap();
        if memory_string.is_empty() {
            let mut memory = ScreepsMemory {
                mem_version: MEMORY_VERSION,
                rooms: HashMap::new(),
                creeps: HashMap::new(),
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
                        mem_version: MEMORY_VERSION,
                        rooms: HashMap::new(),
                        creeps: HashMap::new(),
                        spawn_tick: true,
                    }
                }
            }
        }
    }

    pub fn write_memory(&mut self) {
        let starting_cpu = game::cpu::get_used();
        let serialized = serde_json::to_string(&self).unwrap();
        let js_serialized = JsString::from(serialized);

        screeps::raw_memory::set(&js_serialized);
    }

    pub fn create_creep(&mut self, room_name: &str, creep_name: &str, role: Role) {
        let creep = CreepMemory {
            p: None,
            o_r: room_name.to_string(),
            r: role,
            n_e: None
        };
        self.creeps.insert(creep_name.to_string(), creep);
        info!("Created creep");
    }

    pub fn create_room(&mut self, name: &str) {
        self.rooms.insert(
            name.to_string(),
            RoomMemory {
                name: name.to_string(),
                init: false,
                sources: HashMap::new(),
                creeps: Vec::new(),
            },
        );
    }

    pub fn get_room(&mut self, name: &RoomName) -> &mut RoomMemory {
        self.rooms.get_mut(&name.to_string()).expect("Failed to get room from memory, attempted room name")
    }

    pub fn get_creep(&mut self, name: &str) -> &mut CreepMemory {
        self.creeps.get_mut(&name.to_string()).unwrap()
    }
}

impl RoomMemory {

}