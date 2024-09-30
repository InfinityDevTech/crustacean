use std::collections::HashMap;

use screeps::{game, Direction, PolyStyle, Position, RoomName, RoomXY};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn visualise_path(path: Vec<Position>, _from_pos: Position, color: &str) {
    let style = PolyStyle::default()
    .stroke(color)
    .line_style(screeps::LineDrawStyle::Dashed);

    let mut points_for_room: HashMap<RoomName, Vec<(f32, f32)>> = HashMap::new();

    for step in path {
        let points = (step.x().u8() as f32, step.y().u8() as f32);

        if let Some(existing_points) = points_for_room.get_mut(&step.room_name()) {
            existing_points.push(points);
        } else {
            points_for_room.insert(step.room_name(), vec![points]);
        }
    }

    for (room_name, points) in points_for_room {
        let room = game::rooms().get(room_name);

        if room.is_none() {
            continue;
        }

        let room = room.unwrap();

        room.visual().poly(points, Some(style.clone()));
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn num_to_dir(num: u8) -> Direction {
    match num {
        1 => Direction::Top,
        2 => Direction::TopRight,
        3 => Direction::Right,
        4 => Direction::BottomRight,
        5 => Direction::Bottom,
        6 => Direction::BottomLeft,
        7 => Direction::Left,
        8 => Direction::TopLeft,
        _ => Direction::Top,
    }
}

pub fn dir_to_other_coord(source: RoomXY, dest: RoomXY) -> Direction {
    let x_diff = dest.x.u8() as i8 - source.x.u8() as i8;
    let y_diff = dest.y.u8() as i8 - source.y.u8() as i8;

    match (x_diff, y_diff) {
        (0, -1) => Direction::Top,
        (1, -1) => Direction::TopRight,
        (1, 0) => Direction::Right,
        (1, 1) => Direction::BottomRight,
        (0, 1) => Direction::Bottom,
        (-1, 1) => Direction::BottomLeft,
        (-1, 0) => Direction::Left,
        (-1, -1) => Direction::TopLeft,
        _ => Direction::Top,
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn dir_to_coords(dir: Direction, x: u8, y: u8) -> (u8, u8) {
    match dir {
        Direction::Top => (x, y - 1),
        Direction::TopRight => (x + 1, y - 1),
        Direction::Right => (x + 1, y),
        Direction::BottomRight => (x + 1, y + 1),
        Direction::Bottom => (x, y + 1),
        Direction::BottomLeft => (x - 1, y + 1),
        Direction::Left => (x - 1, y),
        Direction::TopLeft => (x - 1, y - 1),
    }
}