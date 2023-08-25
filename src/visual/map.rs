use std::str::FromStr;

use screeps::{RoomVisual, RoomName, TextStyle};

use crate::memory::ScreepsMemory;

pub fn classify_rooms(memory: &ScreepsMemory) {
    for name in memory.rooms.keys() {
        RoomVisual::new(Some(RoomName::from_str(name).unwrap()))
        .text(2_f32, 2_f32, "Nerd".to_string(), Some(TextStyle::default().color("#ffffff")))
    }
}