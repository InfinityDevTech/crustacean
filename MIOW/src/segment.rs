use js_sys::JsString;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MIOWSegment {
    pub terminal_room: Option<String>,
    pub last_updated: u32,
    pub encrypted_data: String,
}

impl MIOWSegment {
    pub fn new() -> MIOWSegment {
        MIOWSegment {
            terminal_room: None,
            last_updated: 0,
            encrypted_data: String::new(),
        }
    }

    pub fn deserialize_from_js_string(string: JsString) -> Option<MIOWSegment> {
        let my_str = string.as_string();

        if let Some(memory) = my_str {
            if let Ok(memory) = serde_json::from_str(&memory) {
                return Some(memory);
            }
        }

        None
    }
}

pub fn can_decrypt(segment: &MIOWSegment) -> bool {
    let memory = crate::memory::CURRENT_MEMORY.lock().unwrap();
    let memory = memory.as_ref().unwrap();
    let encryption_key = &memory.encryption_key;

    if *encryption_key == String::new() {
        return false;
    }

    let encrypted_data = &segment.encrypted_data;

    let decrypted_data = simple_crypt::decrypt(encrypted_data.as_bytes(), encryption_key.as_bytes()).unwrap();
    let decrypted_data = String::from_utf8(decrypted_data).unwrap();

    let json_value: Result<Value, serde_json::Error> = serde_json::from_str(&decrypted_data);

    if json_value.is_err() {
        return false;
    }

    json_value.unwrap()["check"] == "MIOW"
}