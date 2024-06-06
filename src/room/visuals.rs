use log::info;
use screeps::{game, CircleStyle, HasPosition, MapTextStyle, Position, Room, RoomCoordinate, TextStyle};

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
                format!("{:.1}", progress),
                Default::default(),
            );
        }
    }
}

pub fn visualise_scouted_rooms(memory: &mut ScreepsMemory) {
    for room in memory.scouted_rooms.values() {
        let circle_x = RoomCoordinate::new(46).unwrap();
        let circle_y = RoomCoordinate::new(3).unwrap();

        let text_x = RoomCoordinate::new(3).unwrap();
        let text_y = RoomCoordinate::new(3).unwrap();

        let circle_position = Position::new(circle_x, circle_y, room.name);
        let text_position = Position::new(text_x, text_y, room.name);

        let circle_style = CircleStyle::default()
            .fill("#00FF00")
            .stroke_width(1.0)
            .radius(2.0);

        let text_style = MapTextStyle::default()
            .font_size(6.0)
            .align(screeps::TextAlign::Left)
            .color("#00FF00")
            .stroke_width(0.5);

        screeps::visual::MapVisual::circle(circle_position, circle_style);

        let text = game::time() - room.last_scouted;

        screeps::visual::MapVisual::text(text_position, text.to_string(), text_style);
    }
}

pub fn visualise_controller_progress(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let controller = &cache.structures.controller.as_ref().unwrap().controller;
    let progress = (controller.progress().unwrap() as f32 / controller.progress_total().unwrap() as f32) * 100.0;

    room.visual().text(
        controller.pos().x().u8() as f32,
        controller.pos().y().u8() as f32 - 1.0,
        format!("{:.2}%", progress),
        Default::default(),
    );

    room.visual().text(
        controller.pos().x().u8() as f32,
        controller.pos().y().u8() as f32 + 0.25,
        format!("{}", controller.level()),
        Default::default(),
    );
}