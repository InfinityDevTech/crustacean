use screeps::Room;

use crate::{config, room::cache::CachedRoom};

pub fn get_required_energy_storage(room: &Room) -> u32 {
    let mut base_stock = config::ROOM_ENERGY_STOCKPILE;
    let controller = room.controller().unwrap();

    if controller.level() >= 6 {
        base_stock += 90000;
    }

    if controller.level() < 8 {
        let controller_progress = controller.progress().unwrap();
        let controller_needed = controller.progress_total().unwrap();

        base_stock *= 1 - (controller_progress / controller_needed);
    }

    base_stock
}