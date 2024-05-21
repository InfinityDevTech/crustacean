use std::{cmp, collections::HashMap};

use log::error;
use screeps::{game, ObjectId, Resource, ResourceType, RoomName, Source, Structure, StructureLink};
use serde::{Deserialize, Serialize};

use js_sys::JsString;

use crate::{room::hauling::HaulPriorities, MEMORY_VERSION};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<String>,
    // Career
    pub r: Role,
    // Needs Energy?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n_e: Option<bool>,
    // This is miner specific, the ID of the link next to it
    // If this is empty, then the miner is not linked to a link and it will drop resources on the ground
    // If it is, but the link isnt next to it, the miner will clear the link id. If it is, the miner will deposit resources into the link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l_id: Option<u8>,
    // This is a pointer that changes based on the role of the creep
    // Hauler - A reference to the ID of the current haul orders
    // Miner - A reference to the source in the vec of sources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_id: Option<u8>,
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

    pub haul_orders: Vec<pub struct HaulOrder {
        pub id: u128,
        pub priority: HaulPriorities,
        pub target_id: ObjectId<Structure>,
        pub target_type: ResourceType,
        pub responder: Option<String>,
        pub amount: u32,
    }>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<ObjectId<StructureLink>>>,
    // Creeps by role
    pub creeps: Vec<String>,
    pub creeps_manufactured: u128,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsMemory {
        pub mem_version: u8,
        pub rooms: HashMap<String, RoomMemory>,
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

    pub fn create_creep(&mut self, room_name: &str, creep_name: &str, object: &CreepMemory) {
        self.creeps.insert(creep_name.to_string(), object.clone());

        let room = self.get_room_mut(&RoomName::new(room_name).unwrap());
        room.creeps.push(creep_name.to_string());
    }

    pub fn create_room(&mut self, name: &RoomName, object: &RoomMemory) {
        self.rooms.insert(
            name.to_string(),
            object.clone()
        );
    }

    pub fn get_room_mut(&mut self, name: &RoomName) -> &mut RoomMemory {
        self.rooms.get_mut(&name.to_string()).expect("Failure to resolve room in memory.")
    }
    pub fn get_creep_mut(&mut self, name: &str) -> &mut CreepMemory {
        self.creeps.get_mut(name).expect("Failure to resolve creep in memory.")
    }

    pub fn get_room(&self, name: &RoomName) -> RoomMemory {
        self.rooms.get(&name.to_string()).expect("Failure to resolve room in memory.").clone()
    }
    pub fn get_creep(&self, name: &str) -> CreepMemory {
        self.creeps.get(name).expect("Failure to resolve in memory.").clone()
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