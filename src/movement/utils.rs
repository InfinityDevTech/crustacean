use screeps::{Direction, game, RoomName, PolyStyle, RectStyle};

pub fn visualise_path(path: String, room_name: String, starting_pos: (f32, f32)) {
    let room = game::rooms()
        .get(RoomName::new(&room_name).unwrap());

    if room.is_none() { return; }
    let room = room.unwrap();
    
    let room_vis = room.visual();
    let mut points = vec![];
    let mut cursor = starting_pos;

    for step in path.split("") {
        if step.is_empty() || step == " " {
            continue;
        }
        let dir = num_to_dir(step.parse::<u8>().unwrap());
        points.push((cursor.0, cursor.1));
        let (x, y) = dir_to_coords(dir, cursor.0 as u8, cursor.1 as u8);
        cursor = (x as f32, y as f32);
    }
    points.push((cursor.0, cursor.1));
    room_vis.poly(
        points,
        Some(
            PolyStyle::default()
                .stroke("#ff0000")
                .line_style(screeps::LineDrawStyle::Dashed),
        ),
    );
    room_vis.rect(
        cursor.0 - 0.5,
        cursor.1 - 0.5,
        1.0,
        1.0,
        Some(
            RectStyle::default()
                .stroke("#ff0000")
                .fill("#ff0000")
                .opacity(0.2),
        ),
    );
}

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