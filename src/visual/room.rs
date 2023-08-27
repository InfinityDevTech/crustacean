use std::str::FromStr;

use screeps::{control, game, HasPosition, RoomName, RoomVisual, TextStyle};

use crate::memory::ScreepsMemory;

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

            let controller = room.controller().unwrap();
            roomvis.text(
                controller.pos().x().u8() as f32,
                (controller.pos().y().u8() + 1) as f32,
                format!(
                    "% {:.2}",
                    (controller.progress() * 100) / controller.progress_total()
                ),
                Some(
                    TextStyle::default()
                        .color("#ffffff")
                        .align(screeps::TextAlign::Center),
                ),
            );
        }
    }
}
