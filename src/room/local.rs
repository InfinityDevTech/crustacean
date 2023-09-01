use screeps::{Room, find, HasPosition};

use crate::memory::RoomMemory;

pub fn run_rom(room: Room, _roommem: RoomMemory) {
    let controller = room.controller().unwrap();
    match controller.sign() {
        Some(sign) => {
            if sign.text() != "Ferris FTW!" {
                if let Some(_creep) = controller.pos().find_closest_by_range(find::MY_CREEPS) {
                }
            }
        },
        None => todo!(),
    }
}