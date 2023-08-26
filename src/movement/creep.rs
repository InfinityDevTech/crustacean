use log::info;
use screeps::{game, Direction, HasPosition, Position};

use crate::memory::CreepMemory;

use super::move_target::MoveTarget;

pub fn move_to(creep_name: &String, creep_memory: &mut CreepMemory, target: Position) {
    let creep = game::creeps().get(creep_name.to_string()).unwrap();
    match &creep_memory.p {
        Some(path) => {
            move_by_path(creep_name.to_string(), path.clone(), creep_memory)
        }
        None => {
            let target = MoveTarget {
                pos: target,
                range: 1,
            }.find_path_to(creep.pos());
            creep_memory.p = Some(target.clone());
            move_by_path(creep_name.to_string(), target, creep_memory);
        }

    }
}

pub fn move_by_path(creep_name: String, path: String, memory: &mut CreepMemory) {
    let creep = game::creeps().get(creep_name).unwrap();

    if creep.fatigue() > 0 {
        return;
    }
    let serialized_path = path;
    let serialized_vec = serialized_path.split("").filter(|x| x != &"").map(|x| x.parse::<u8>().unwrap()).collect::<Vec<u8>>();
    let step_dir = num_to_dir(serialized_vec[0]);

    match creep.move_direction(step_dir) {
        Ok(_) => {},
        Err(e) => info!("Creep move failed, {:?}", e),
    };

    let serialized_vec = serialized_vec[1..].to_vec();
    let serialized_path = serialized_vec.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("");
    if serialized_vec.is_empty() {
        memory.p = None
    } else {
        memory.p = Some(serialized_path);
    }

    let mut points = vec![];
    let mut cursor = (creep.pos().x().u8() as f32, creep.pos().y().u8() as f32);
    for step in serialized_vec {
        let dir = num_to_dir(step);
        let (x, y) = dir_to_coords(dir, cursor.0, cursor.1);
        points.push((x, y));
        cursor = (x, y);
    }
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

pub fn dir_to_coords(dir: Direction, x: f32, y: f32) -> (f32, f32) {
    match dir {
        Direction::Top => (x, y - 1_f32),
        Direction::TopRight => (x + 1_f32, y - 1_f32),
        Direction::Right => (x + 1_f32, y),
        Direction::BottomRight => (x + 1_f32, y + 1_f32),
        Direction::Bottom => (x, y + 1_f32),
        Direction::BottomLeft => (x - 1_f32, y + 1_f32),
        Direction::Left => (x - 1_f32, y),
        Direction::TopLeft => (x - 1_f32, y - 1_f32),
    }
}
