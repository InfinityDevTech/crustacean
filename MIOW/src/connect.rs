use std::{str::FromStr, sync::Mutex};

use js_sys::JsString;
use screeps::{game, raw_memory, ResourceType};
use wasm_bindgen::prelude::*;

use crate::{memory::{self, CURRENT_MEMORY}, segment::{can_decrypt, MIOWSegment}, terminal, MASTER};

static NEEDS_REFRESH: Mutex<bool> = Mutex::new(false);
static CONNECTION_PHASE: Mutex<u32> = Mutex::new(1);

#[wasm_bindgen(js_name = connect_MIOW)]
pub fn connect() {
    let memory = memory::CURRENT_MEMORY.lock().unwrap();
    let mut needs_refresh = NEEDS_REFRESH.lock().unwrap();
    let mut connection_phase = CONNECTION_PHASE.lock().unwrap();

    if *needs_refresh {
        match *connection_phase {
            1 => connection_phase_1(),
        }
    }
}

fn connection_phase_1() {
    let memory = memory::CURRENT_MEMORY.lock().unwrap();
    let mut needs_refresh = NEEDS_REFRESH.lock().unwrap();
    let mut connection_phase = CONNECTION_PHASE.lock().unwrap();

    let owner_segment = raw_memory::foreign_segment();

    // If we have the owner segment, get it, if not, we need to refresh
    if let Some(owner_segment) = owner_segment {
        // If its the owners, we can decrypt it
        if owner_segment.username() == JsString::from_str(MASTER).unwrap() {
            // If we can deserialize it, we can decrypt it
            if let Some(segment_data) = MIOWSegment::deserialize_from_js_string(owner_segment.data()) {
                // If we can decrypt it (valid key), we can move on, if not, we need to go to step 2
                if can_decrypt(&segment_data) {
                    *needs_refresh = false;
                    *connection_phase = 2;
                } else {
                    *needs_refresh = true;
                    *connection_phase = 2;
                }
            } else {
                *needs_refresh = true;
                *connection_phase = 1;
            }
        }
    } else {
        *needs_refresh = true;
        *connection_phase = 1;
    }
}

fn connection_phase_2() {
    let memory = memory::CURRENT_MEMORY.lock().unwrap();
    let memory = memory.as_ref().unwrap();
    let mut needs_refresh = NEEDS_REFRESH.lock().unwrap();
    let mut connection_phase = CONNECTION_PHASE.lock().unwrap();

    let owner_segment = raw_memory::foreign_segment();

    // If we have the owner segment, get it, if not, we need to refresh
    if let Some(owner_segment) = owner_segment {
        // If its the owners, we can decrypt it
        if owner_segment.username() == JsString::from_str(MASTER).unwrap() {
            // If we can deserialize it, we can decrypt it
            if let Some(segment_data) = MIOWSegment::deserialize_from_js_string(owner_segment.data()) {
                let terminal = segment_data.terminal_room;
                let my_terminal = terminal::get_terminal().unwrap();

                if let Some(terminal) = terminal {
                    let cost = game::market::calc_transaction_cost(100, &JsString::from_str(&my_terminal.room().unwrap().name().to_string()).unwrap(), &JsString::from_str(&terminal).unwrap());
                    my_terminal.send(ResourceType::Energy, 1+cost, terminal, description)

                }
            }
        }
    }

    *needs_refresh = true;
                *connection_phase = 1;
}