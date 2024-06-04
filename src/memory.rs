use std::collections::HashMap;

use log::error;
use screeps::{ObjectId, RawObjectId, ResourceType, RoomName, StructureContainer, StructureLink};
use serde::{Deserialize, Serialize};

use js_sys::JsString;

use crate::{config::MEMORY_VERSION, room::cache::tick_cache::hauling::{HaulingPriority, HaulingType}};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Copy)]
// The roles listed in creep memory
// The order of this also is the order in which
// Traffic Priority is handled.
pub enum Role {
    // Mining industry
    Miner = 0,
    Hauler = 1,

    FastFiller = 2,

    // Construction industry
    Upgrader = 3,
    Builder = 4,

    Bulldozer = 5,

    Scout = 10,
}

// What each creep stores in its memory.
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct CreepMemory{
    // Owning room
    #[serde(rename = "0")]
    pub owning_room: String,
    // Path
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "1")]
    pub path: Option<String>,
    // Career
    //#[serde(rename = "2")]
    //pub role: Role,
    // Needs Energy?
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "3")]
    pub needs_energy: Option<bool>,
    // This is miner specific, the ID of the link next to it
    // If this is empty, then the miner is not linked to a link and it will drop resources on the ground
    // If it is, but the link isnt next to it, the miner will clear the link id. If it is, the miner will deposit resources into the link
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "4")]
    pub link_id: Option<ObjectId<StructureLink>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "5")]
    pub fastfiller_container: Option<ObjectId<StructureContainer>>,
    // This is a pointer that changes based on the role of the creep
    // Miner - A reference to the source in the vec of sources
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "6")]
    pub task_id: Option<u128>,
    // The hauling task if a creep is a hauler.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "7")]
    pub hauling_task: Option<pub struct CreepHaulTask {
        #[serde(rename = "0")]
        pub target_id: RawObjectId,
        #[serde(rename = "1")]
        pub haul_type: HaulingType,
        #[serde(rename = "2")]
        pub priority: f32,
        #[serde(rename = "3")]
        pub resource: ResourceType,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "4")]
        pub amount: Option<u32>,
    }>,
}
}

// Room Memory
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct RoomMemory{
    // Name
    pub name: String,
    pub rcl: u8,
    pub id: u128,
    pub planned: bool,
    // Creeps by role
    pub creeps: Vec<String>,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct EnemyPlayer {
        pub username: String,
        pub hate: f32,

        pub owned_rooms: Vec<RoomName>,
        pub reserved_rooms: Vec<RoomName>,

        pub last_attack: u32,
    }
}
// Top level memory.
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsMemory {
        pub mem_version: u8,
        pub rooms: HashMap<RoomName, RoomMemory>,
        pub creeps: HashMap<String, CreepMemory>,

        pub enemy_players: HashMap<String, EnemyPlayer>,
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

                enemy_players: HashMap::new(),
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

                        enemy_players: HashMap::new(),
                    }
                }
            }
        }
    }

    pub fn write_memory(&mut self) {
        let serialized = serde_json::to_string(&self).unwrap();
        let js_serialized = JsString::from(serialized);

        screeps::raw_memory::set(&js_serialized);
    }

    pub fn create_creep(&mut self, room_name: &str, creep_name: &str, object: CreepMemory) {
        self.creeps.insert(creep_name.to_string(), object);

        let room = self.rooms.get_mut(&RoomName::new(room_name).unwrap()).unwrap();
        room.creeps.push(creep_name.to_string());
    }

    pub fn create_room(&mut self, name: &RoomName, object: RoomMemory) {
        self.rooms.insert(
            *name,
            object
        );
    }
}

impl EnemyPlayer {
    pub fn decrement_hate(&mut self, amount: f32) {
        let current_hate = self.hate;

        if current_hate - amount < 0.0 {
            self.hate = 0.0;
        } else {
            self.hate -= amount;
        }
    }

    pub fn increment_hate(&mut self, amount: f32) {
        self.hate += amount;
    }
}