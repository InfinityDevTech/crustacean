use std::str::FromStr;

use screeps::{RoomVisual, RoomName, TextStyle, game};

use crate::memory::ScreepsMemory;

pub fn classify_rooms(memory: &ScreepsMemory) {
    for name in memory.rooms.keys() {
        let roommem = memory.rooms.get(name).unwrap();
        RoomVisual::new(Some(RoomName::from_str(name).unwrap()))
        .text(1_f32, 1_f32, "Stats".to_string(), Some(TextStyle::default().color("#ffffff").align(screeps::TextAlign::Left)));

        RoomVisual::new(Some(RoomName::from_str(name).unwrap()))
        .text(1_f32, 2_f32, format!("Miners: {}", roommem.c_c.miner), Some(TextStyle::default().color("#ffffff").align(screeps::TextAlign::Left)));

        RoomVisual::new(Some(RoomName::from_str(name).unwrap()))
        .text(1_f32, 3_f32, format!("Haulers: {}", roommem.c_c.hauler), Some(TextStyle::default().color("#ffffff").align(screeps::TextAlign::Left)));

        RoomVisual::new(Some(RoomName::from_str(name).unwrap()))
        .text(1_f32, 4_f32, format!("Upgraders: {}", roommem.c_c.upgrader), Some(TextStyle::default().color("#ffffff").align(screeps::TextAlign::Left)));
    }
}