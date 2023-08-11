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
        let memory: Memory = serde_json::from_str(&memory_jsstring.as_string().unwrap()).unwrap();
        return memory;
    }

    pub fn write_memory(&self) {
        screeps::raw_memory::set(&JsString::from(serde_json::to_string(&self).unwrap()));
    }
}
