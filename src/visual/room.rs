use std::str::FromStr;

use screeps::{game, HasPosition, RoomName, RoomVisual, TextStyle, MapVisual, Position, RoomCoordinate};

use crate::{memory::ScreepsMemory, traits::room::RoomExtensions};

pub fn classify_rooms(memory: &ScreepsMemory) {
    for name in memory.rooms.keys() {
        let roommem = memory.rooms.get(name).unwrap();
        if let Some(room) = game::rooms().get(RoomName::from_str(name).unwrap()) {
            let roomvis = RoomVisual::new(Some(RoomName::from_str(name).unwrap()));
            let white_left = Some(
                TextStyle::default()
                    .color("#ffffff")
                    .align(screeps::TextAlign::Left),
            );
            // Creep counters
            roomvis.text(1_f32, 1_f32, "Stats".to_string(), white_left.clone());
            roomvis.text(
                1_f32,
                2_f32,
                format!("Miners: {}", roommem.c_c.miner),
                white_left.clone(),
            );
            roomvis.text(
                1_f32,
                3_f32,
                format!("Haulers: {}", roommem.c_c.hauler),
                white_left.clone(),
            );
            roomvis.text(
                1_f32,
                4_f32,
                format!("Upgraders: {}", roommem.c_c.upgrader),
                white_left.clone(),
            );
            roomvis.text(
                1_f32,
                5_f32,
                format!("Builders: {}", roommem.c_c.builder),
                white_left.clone(),
            );

            let controller = room.controller().unwrap();
            roomvis.text(
                controller.pos().x().u8() as f32,
                (controller.pos().y().u8() - 1) as f32,
                format!(
                    "% {:.2}",
                    controller.progress() as f64 / controller.progress_total() as f64 * 100.0
                ),
                Some(
                    TextStyle::default()
                        .color("#ffffff")
                        .align(screeps::TextAlign::Center),
                ),
            );
        }
    }

    for room in game::rooms().values() {
        let pos = Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(3).unwrap(), room.name());
        MapVisual::text(pos, get_room_type(&room.name_str()), Some(TextStyle::default().color("#ffffff").align(screeps::TextAlign::Center)));
    }
}

pub fn get_room_type(name: &str) -> String {
    if let Some(room) = game::rooms().get(RoomName::from_str(name).unwrap()) {
        if room.is_highway() {
            "highway".to_string()
        } else if room.is_intersection() {
            return "intersection".to_string();
        } else if room.is_source_keeper() {
            return "source_keeper".to_string();
        } else {
            return "normal".to_string();
        }
    } else {
        "none".to_string()
    }
}