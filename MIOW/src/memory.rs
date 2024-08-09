use std::{collections::HashMap, sync::Mutex};

use js_sys::{JsString, Map};
use screeps::RawObjectId;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::segment::MIOWSegment;

pub static CURRENT_MEMORY: Mutex<Option<MIOWMemory>> = Mutex::new(None);

#[derive(Serialize, Deserialize, Clone)]
pub struct MIOWMemory {
    pub terminal_room: String,
    pub encryption_key: String,
    pub known_segments: HashMap<String, MIOWSegment>,
}

impl MIOWMemory {
    pub fn setup(string: JsString) -> MIOWMemory {
        let my_str = string.as_string();

        let memory = if let Some(memory) = my_str {
            if let Ok(memory) = serde_json::from_str(&memory) {
                memory
            } else {
                MIOWMemory::new()
            }
        } else {
            MIOWMemory::new()
        };

        let mut current_memory = CURRENT_MEMORY.lock().unwrap();
        *current_memory = Some(memory.clone());

        memory
    }

    pub fn new() -> MIOWMemory {
        MIOWMemory {
            terminal_room: String::new(),
            encryption_key: String::new(),
            known_segments: HashMap::new(),
        }
    }
}