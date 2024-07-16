use std::cmp;

use log::info;
use screeps::{Direction, Position, RoomName};

use crate::{constants::WORLD_SIZE, traits::room::RoomNameExtensions};


#[derive(Clone, Debug, Default)]
pub struct MapPosition {
    pub xx: u8,
    pub yy: u8,
}

impl MapPosition {
    pub fn id(&self) -> u16 {
        (self.xx as u16) << 8 | self.yy as u16
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct WorldPosition {
    pub xx: u32,
    pub yy: u32,
}

pub fn null_world_pos() -> WorldPosition {
    WorldPosition { xx: u32::MAX, yy: u32::MIN }
}

pub fn world_position(xx: u32, yy: u32) -> WorldPosition {
    if xx >= (u32::MAX) / 100 || yy >= (u32::MAX) / 100 {
        info!("Detecting overflow: XX: {}, YY: {}", xx, yy);
    }
    WorldPosition { xx, yy }
}

impl WorldPosition {
    pub fn is_null(&self) -> bool {
        self.xx == u32::MAX && self.yy == u32::MIN
    }

    pub fn range_to(&self, pos: &WorldPosition) -> u32 {
        let other = if pos.xx > self.xx {
            pos.xx - self.xx
        } else {
            self.xx - pos.xx
        };

        let otherr = if pos.yy > self.yy {
            pos.yy - self.yy
        } else {
            self.yy - pos.yy
        };

        cmp::max(other, otherr)
    }

    pub fn position_to_dir(&self, dir: Direction) -> WorldPosition {
        match dir {
            Direction::Top => WorldPosition { xx: self.xx, yy: self.yy - 1 },
            Direction::TopRight => WorldPosition { xx: self.xx + 1, yy: self.yy - 1 },
            Direction::Right => WorldPosition { xx: self.xx + 1, yy: self.yy },
            Direction::BottomRight => WorldPosition { xx: self.xx + 1, yy: self.yy + 1 },
            Direction::Bottom => WorldPosition { xx: self.xx, yy: self.yy + 1 },
            Direction::BottomLeft => WorldPosition { xx: self.xx - 1, yy: self.yy + 1 },
            Direction::Left => WorldPosition { xx: self.xx - 1, yy: self.yy },
            Direction::TopLeft => WorldPosition { xx: self.xx - 1, yy: self.yy - 1 }
        }
    }

    pub fn direction_to(&self, pos: &WorldPosition) -> Direction {
        let dx = pos.xx as i32 - self.xx as i32;
        let dy = pos.yy as i32 - self.yy as i32;

        if dx > 0 {
            if dy > 0 {
                return Direction::BottomRight;
            } else if dy < 0 {
                return Direction::TopRight;
            } else {
                return Direction::Right;
            }
        } else if dx < 0 {
            if dy > 0 {
                return Direction::BottomLeft;
            } else if dy < 0 {
                return Direction::TopLeft;
            } else {
                return Direction::Left;
            }
        } else if dy > 0 {
            return Direction::Bottom;
        } else if dy < 0 {
            return Direction::Top;
        }

        return Direction::Top;
    }

    pub fn map_position(&self) -> MapPosition {
        MapPosition {xx: (self.xx / 50) as u8, yy: (self.yy / 50) as u8 }
    }
}

pub fn generate_room_name(xx: u8, yy: u8) -> String {
    let p1 = if xx <= (WORLD_SIZE >> 1) {
        format!("W{}", (WORLD_SIZE >> 1) - xx)
    } else {
        format!("E{}", xx - (WORLD_SIZE >> 1) - 1)
    };

    let p2 = if yy <= (WORLD_SIZE >> 1) {
        format!("N{}", (WORLD_SIZE >> 1) - yy)
    } else {
        format!("S{}", yy - (WORLD_SIZE >> 1) - 1)
    };

    format!("{}{}", p1, p2)
}

pub fn parse_room_name(name: &RoomName) -> (i64, i64) {
    let (xname, xx, yname, yy) = name.split_name();
    let xx = xx as i64;
    let yy = yy as i64;

    let rx = (WORLD_SIZE >> 1) as i64 + if xname == "W" { -xx } else { xx + 1 };
    let ry = (WORLD_SIZE >> 1) as i64 + if yname == "N" { -yy } else { yy + 1 };

    if !(rx >= 0 && rx < WORLD_SIZE as i64 && ry >= 0 && ry < WORLD_SIZE as i64) {
        panic!("Invalid room name: {:?}", name);
    }

    (rx, ry)
}

pub fn position_to_world_position(pos: &Position) -> WorldPosition {
    let xx = pos.x().u8() as u32;
    let yy = pos.y().u8() as u32;

    if !((0..50).contains(&xx) && (0..50).contains(&yy)) {
        panic!("Invalid position: {:?}", pos);
    }

    let (xoffset, yoffset) = parse_room_name(&pos.room_name());

    world_position(
        xx + (xoffset as u32) * 50,
        yy + (yoffset as u32) * 50,
    )
}