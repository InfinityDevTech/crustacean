use std::{any::Any, str::FromStr, sync::Mutex};

use js_sys::{wasm_bindgen, JsString, Object};
use log::*;
use screeps::{
    game, raw_memory, HasId, ObjectId, OwnedStructureProperties, StructureObject, StructureProperties, StructureTerminal, StructureType
};
use wasm_bindgen::prelude::*;

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();
static TERMINAL_TO_USE: Mutex<Option<ObjectId<StructureTerminal>>> = Mutex::new(None);
static RUN_CHECK_TIME: Mutex<u32> = Mutex::new(0);
static CAN_RUN: Mutex<bool> = Mutex::new(false);

static MASTER: &str = "DroidFreak36";

pub mod connect;
pub mod logging;
pub mod memory;
pub mod segment;
pub mod terminal;

#[wasm_bindgen(js_name = getMIOWsegment)]
pub fn get_segment() -> u8 {
    99
}

#[wasm_bindgen(js_name = initialise_MIOW)]
pub fn initialise(memory: JsString) {
    INIT_LOGGING.call_once(|| {
        // show all output of Info level, adjust as needed
        logging::setup_logging(LevelFilter::Info);
    });

    info!("[MIOW] Initialising MIOW");
    let can_run = can_run();

    if !can_run {
        return;
    }

    memory::MIOWMemory::setup(memory);
    raw_memory::set_active_segments(&[get_segment()]);
}

// Checks if we can run by doing the following
// if CAN_RUN is false, and the last check was more than 100 ticks ago, check if we have a terminal
//   if we have a terminal, set CAN_RUN to true
// if CAN_RUN is true, and the last check was more than 1k ticks ago, do the same.
fn can_run() -> bool {
    let mut can_run = CAN_RUN.lock().unwrap();
    let mut run_check_time = RUN_CHECK_TIME.lock().unwrap();
    let current_time = game::time();

    if !*can_run && current_time > *run_check_time + 100 {
        let terminal = terminal::get_terminal();

        if terminal.is_none() {
            info!("[MIOW] No terminal found, we are not letting you run for the next 100 ticks to not consume CPU.");
            *run_check_time = current_time + 100;
            return false;
        }

        *can_run = true;
        info!("[MIOW] Terminal found, we are now letting you run.");
        return true;
    }

    if *can_run && current_time > *run_check_time + 1000 {
        let terminal = terminal::get_terminal();

        if terminal.is_none() {
            info!("[MIOW] No terminal found, we are not letting you run for the next 100 ticks to not consume CPU.");
            *run_check_time = current_time + 100;
            *can_run = false;
            return false;
        }
    }

    *can_run
}