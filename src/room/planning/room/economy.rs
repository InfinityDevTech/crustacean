use screeps::Room;

use crate::config;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_required_energy_storage(room: &Room) -> u32 {
    let mut base_stock = config::ROOM_ENERGY_STOCKPILE;
    let controller = room.controller().unwrap();

    // Unplug test

    if controller.level() >= 6 {
        base_stock += 90000;
    }

    if controller.level() < 8 {
        let controller_progress = controller.progress().unwrap_or(0);
        let controller_needed = controller.progress_total().unwrap_or(0);

        if controller_progress > 0 && controller_needed > 0 {
            base_stock *= 1 - (controller_progress / controller_needed);
        }
    }

    base_stock
}