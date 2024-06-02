use screeps::{HasPosition, Room, TextStyle};

use crate::memory::ScreepsMemory;

use super::cache::tick_cache::RoomCache;

pub fn run_full_visuals(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    visualise_spawn_progess(room, memory, cache);
    visualise_controller_progress(room, memory, cache);
}

pub fn visualise_spawn_progess(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    for spawn in cache.structures.spawns.values() {
        if let Some(spawning) = spawn.spawning() {
            let progress = (spawning.remaining_time() as f32 / spawning.need_time() as f32) * 100.0;

            room.visual().text(
                spawn.pos().x().u8() as f32,
                spawn.pos().y().u8() as f32 + 0.25,
                format!("{}%", progress.round() as u32),
                Default::default(),
            );
        }
    }
}

pub fn visualise_controller_progress(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let controller = &cache.structures.controller.as_ref().unwrap().controller;
    let progress = (controller.progress().unwrap() as f32 / controller.progress_total().unwrap() as f32) * 100.0;

    room.visual().text(
        controller.pos().x().u8() as f32,
        controller.pos().y().u8() as f32 - 1.0,
        format!("{}%", progress.round() as u32),
        Default::default(),
    );

    room.visual().text(
        controller.pos().x().u8() as f32,
        controller.pos().y().u8() as f32 + 0.25,
        format!("{}", controller.level()),
        Default::default(),
    );
}