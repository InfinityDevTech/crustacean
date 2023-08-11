#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use js_sys::JsString;

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct CreepMemory{
    pub _move: Option<pub struct {
        pub dest: struct {
        pub x: i32,
        pub y: i32,
        pub room: String
    },
    pub time: i32,
    pub path: String,
    pub room: String
    }>,
    pub work: Option<crate::CreepTarget>,
}
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct Memory {
        pub creeps: HashMap<String, CreepMemory>
}
}

impl Memory {
    pub fn init_memory() -> Self {
        let memory_jsstring = screeps::raw_memory::get();
        let memory_string = memory_jsstring.as_string().unwrap();
        if memory_string == "" {
            let memory = Memory {
                creeps: HashMap::new(),
            };
            memory.write_memory();
            memory
        } else {
            let memory: Memory = serde_json::from_str(&memory_string).unwrap();
            memory
        }
    }

    pub fn write_memory(&self) {
        //let serialized = serde_json::to_string(&self).unwrap();
        //et js_serialized = JsString::from(serialized);
        //screeps::raw_memory::set(&js_serialized);
    }
}
