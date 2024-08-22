use screeps::{game, CircleStyle, HasPosition, MapTextStyle, Position, Room, RoomCoordinate, RoomName, TextStyle};

use crate::{allies, config, memory::ScreepsMemory};

use super::cache::CachedRoom;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_full_visuals(room: &Room, memory: &mut ScreepsMemory, cached_room: &mut CachedRoom) {
    visualise_spawn_progess(room, memory, cached_room);
    visualise_controller_progress(room, memory, cached_room);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn visualise_spawn_progess(room: &Room, _memory: &mut ScreepsMemory, cache: &mut CachedRoom) {
    for spawn in cache.structures.spawns.values() {
        if let Some(spawning) = spawn.spawning() {
            room.visual().text(
                spawn.pos().x().u8() as f32,
                spawn.pos().y().u8() as f32 + 0.25,
                format!("{:.1}", spawning.remaining_time() - 1),
                Some(TextStyle::default().align(screeps::TextAlign::Center)),
            );
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn visualise_scouted_rooms(memory: &mut ScreepsMemory) {
    if config::VISUALISE_SCOUTING_DATA {
        for (room_name, room) in &memory.scouted_rooms {
            let circle_x = RoomCoordinate::new(46).unwrap();
            let circle_y = RoomCoordinate::new(3).unwrap();

            let text_x = RoomCoordinate::new(3).unwrap();
            let text_y = RoomCoordinate::new(3).unwrap();

            let circle_position = Position::new(circle_x, circle_y, room.name);
            let text_position = Position::new(text_x, text_y, room.name);

            let potential_owner = room
                .owner
                .clone()
                .unwrap_or_else(|| room.reserved.clone().unwrap_or("None".to_string()));

            let circle_style = if !allies::is_ally(&room.owner.clone().unwrap_or("".to_string()), Some(*room_name)) {
                CircleStyle::default()
                    .fill("#00FF00")
                    .stroke_width(1.0)
                    .radius(2.0)
            } else {
                CircleStyle::default()
                    .fill("#FF0000")
                    .stroke_width(1.0)
                    .radius(2.0)
            };

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
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn visualise_room_visual(room: &RoomName) {
    let circle_x = RoomCoordinate::new(46).unwrap();
    let circle_y = RoomCoordinate::new(46).unwrap();

    let pos = Position::new(circle_x, circle_y, *room);

    let style = CircleStyle::default()
    .fill("#0000FF")
    .stroke_width(1.0)
    .radius(2.0);

    screeps::MapVisual::circle(pos, style);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn visualise_controller_progress(
    room: &Room,
    _memory: &mut ScreepsMemory,
    cache: &mut CachedRoom,
) {
    let controller = &cache.structures.controller.as_ref().unwrap();

    if controller.level() == 8 || controller.progress().is_none() || controller.progress_total().is_none() {
        return;
    }
    let progress = (controller.progress().unwrap() as f32
        / controller.progress_total().unwrap() as f32)
        * 100.0;

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
        Some(TextStyle::default().align(screeps::TextAlign::Center).color("#FFFFFF")),
    );
}
