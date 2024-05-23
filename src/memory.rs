use std::{cmp, collections::HashMap};

use log::error;
use screeps::{game, ObjectId, RawObjectId, ResourceType, RoomName, Source, StructureLink};
use serde::{Deserialize, Serialize};

use js_sys::JsString;

use crate::{room::cache::hauling::{HaulingPriority, HaulingType}, MEMORY_VERSION};

pub const ALLIES: [&str; 2] = ["MarvinTMB", "Tigga"];

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Copy)]
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
    #[serde(rename = "0")]
    pub owning_room: String,
    // Path
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "1")]
    pub path: Option<String>,
    // Career
    #[serde(rename = "2")]
    pub role: Role,
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
    // This is a pointer that changes based on the role of the creep
    // Miner - A reference to the source in the vec of sources
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "5")]
    pub task_id: Option<u128>,
    // The hauling task if a creep is a hauler.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "6")]
    pub hauling_task: Option<pub struct CreepHaulTask {
        #[serde(rename = "0")]
        pub target_id: RawObjectId,
        #[serde(rename = "1")]
        pub haul_type: HaulingType,
        #[serde(rename = "2")]
        pub priority: HaulingPriority,
        #[serde(rename = "3")]
        pub resource: ResourceType,
        #[serde(rename = "4")]
        pub amount: u32,
    }>,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct RoomMemory{
    // Name
    pub name: String,
    pub id: u128,
    // Mining stuffs
    pub sources: Vec<pub struct ScoutedSource {
        pub id: ObjectId<Source>,
        pub assigned_creeps: u8,
        pub max_creeps: u8,
        pub work_parts: u8,
    }>,
    // Creeps by role
    pub creeps: Vec<String>,
    pub creeps_manufactured: u128,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsMemory {
        pub mem_version: u8,
        pub rooms: HashMap<RoomName, RoomMemory>,
        pub creeps: HashMap<String, CreepMemory>
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

impl ScoutedSource {
    pub fn parts_needed(&self) -> u8 {
        let source: Source = game::get_object_by_id_typed(&self.id).unwrap();
        let max_energy = source.energy_capacity();

        // Each work part equates to 2 energy per tick
        // Each source refills energy every 300 ticks.
        let max_work_needed = (max_energy / 300) + 2;

        let work_parts_needed = max_work_needed - self.work_parts as u32;

        cmp::max(work_parts_needed, 0) as u8
    }
}