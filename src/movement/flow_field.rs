use std::u8;

use log::info;
use screeps::{Direction, LineStyle, LocalCostMatrix, Position, Room, RoomCoordinate, RoomXY, Terrain, TextStyle};

use crate::heap_cache::compressed_matrix::CompressedMatrix;

use super::movement_utils::{dir_to_coords, num_to_dir};

pub struct FlowFieldSource {
    pub pos: RoomXY,
    pub cost: u8,
}

pub struct FlowField {
    pub data: Vec<u8>,
    pub directions: Option<Vec<u8>>,
    pub width: u8,
    pub height: u8,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl FlowField {
    pub fn new(width: u8, height: u8, directions: bool) -> FlowField {
        let arr_size = width as usize * height as usize;

        let directions = if directions {
            Some(vec![u8::MAX; arr_size])
        } else {
            None
        };

        let data = vec![u8::MAX; arr_size];

        FlowField {
            data,
            directions,
            width,
            height,
        }
    }

    pub fn generate(&mut self, sources: Vec<FlowFieldSource>, cost_callback: impl Fn() -> LocalCostMatrix, max_cost: Option<u8>) -> CompressedMatrix {
        let distance_max_cost = max_cost.unwrap_or(u8::MAX);
        let mut cm = cost_callback();

        let mut queue = Vec::new();
        for source in sources {
            self.data[(source.pos.y.u8() as u16 * self.width as u16 + source.pos.x.u8() as u16) as usize] = 0;

            // score used to be cm.get(source.pos), but if the CM score was greater than 0, it would integer
            // overflow when doing distance math. Hooplah.
            queue.push((source.pos, 0));
        }

        while !queue.is_empty() {
            let len = queue.len();
            for pos in 0..len {
                let queue_pos = queue[pos].0;
                let score = queue[pos].1;

                let pos_is_exit = is_exit(queue_pos.x.u8(), queue_pos.y.u8());
                let next_to_exit = is_next_to_exit(queue_pos.x.u8(), queue_pos.y.u8());
                let distance = self.data[(queue_pos.y.u8() as u16 * self.width as u16 + queue_pos.x.u8() as u16) as usize] as u16 + score as u16;

                for dir in 1..=8 {
                    let dir = num_to_dir(dir);
                    let (dx, dy) = dir_to_coords(dir, queue_pos.x.u8(), queue_pos.y.u8());

                    if dx >= 50 || dy >= 50 {
                        continue;
                    }

                    let cm_res = cm.get(RoomXY::new(RoomCoordinate::new(dx).unwrap(), RoomCoordinate::new(dy).unwrap()));
                    if cm_res == 255 {
                        continue;
                    }

                    let index = (dy as u16 * self.width as u16 + dx as u16) as usize;
                    if distance < self.data[index] as u16 {
                        self.data[index] = distance as u8;

                        if let Some(ref mut directions) = self.directions {
                            directions[index] = (dir as u8 + 3) % 8 + 1;
                        }

                        if distance as u8 <= distance_max_cost {
                            queue.push((RoomXY::new(RoomCoordinate::new(dx).unwrap(), RoomCoordinate::new(dy).unwrap()), cm_res));
                        }
                    }
                }
            }

            queue.drain(0..len);
        }

        let mut cdm = CompressedMatrix::new();

        for x in 0..self.width {
            for y in 0..self.height {
                let index = (y as u16 * self.width as u16 + x as u16) as usize;
                let mut dir = self.directions.as_ref().unwrap()[index];

                let (dx, dy) = dir_to_coords(num_to_dir(dir), x, y);
                let pointing_to_exit = is_next_to_exit(dx, dy);

                if dx >= 50 || dy >= 50 || dir > 8 {
                    cdm.set_xy(x, y, dir);

                    continue;
                }

                // If we are an exit, we need to point to a non-exit
                if pointing_to_exit {
                    let mut lowest_dir = u8::MAX;
                    let mut lowest_score = u8::MAX;

                    for dirs in 1..=8 {
                        let ndir = num_to_dir(dirs);
                        let score = self.data[(dy as u16 * self.width as u16 + dx as u16) as usize];

                        let (dx, dy) = dir_to_coords(ndir, x, y);

                        if !is_exit(dx, dy) && cm.get(RoomXY::new(RoomCoordinate::new(dx).unwrap(), RoomCoordinate::new(dy).unwrap())) < 255 && score < lowest_score {
                            lowest_score = score;
                            lowest_dir = ndir as u8;
                        }
                    }

                    if lowest_dir != u8::MAX {
                        dir = lowest_dir;
                    }
                }

                cdm.set_xy(x, y, dir);
            }
        }

        cdm
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn visualise_field(room: &Room, field: &CompressedMatrix) {
    let vis = room.visual();

    info!("Bisualsign");

    for x in 0..50 {
        for y in 0..50 {
            let dir = field.get_dir(x, y);

            let from_pos = Position::new(RoomCoordinate::new(x).unwrap(), RoomCoordinate::new(y).unwrap(), room.name());

            if dir.is_none() {
                continue;
            }

            vis.text(from_pos.x().u8() as f32, from_pos.y().u8() as f32, dir.unwrap().to_string(), Some(TextStyle::default().align(screeps::TextAlign::Center)));
        }
    }
}

pub fn is_next_to_exit(x: u8, y: u8) -> bool {
    x == 1 || y == 1 || x >= 48 || y >= 48
}

pub fn is_exit(x: u8, y: u8) -> bool {
    x == 0 || y == 0 || x >= 49 || y >= 49
}