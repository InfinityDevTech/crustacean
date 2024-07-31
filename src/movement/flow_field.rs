use std::u8;

use log::info;
use screeps::{Direction, LineStyle, LocalCostMatrix, Position, Room, RoomCoordinate, RoomXY, Terrain, TextStyle};

use crate::heap_cache::CompressedDirectionMatrix;

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

    pub fn generate(&mut self, sources: Vec<FlowFieldSource>, cost_callback: impl Fn() -> LocalCostMatrix, max_cost: Option<u8>) -> CompressedDirectionMatrix {
        let distance_max_cost = max_cost.unwrap_or(u8::MAX);
        let mut cm = cost_callback();

        let mut queue = Vec::new();
        for source in sources {
            self.data[(source.pos.y.u8() as u16 * self.width as u16 + source.pos.x.u8() as u16) as usize] = 0;

            queue.push((source.pos, cm.get(source.pos)));
        }

        while !queue.is_empty() {
            let len = queue.len();
            for pos in 0..len {
                let queue_pos = queue[pos].0;
                let score = queue[pos].1;

                let distance = self.data[(queue_pos.y.u8() as u16 * self.width as u16 + queue_pos.x.u8() as u16) as usize] as u16 + score as u16;

                for dir in 1..=8 {
                    let dir = num_to_dir(dir);
                    let (dx, dy) = dir_to_coords(dir, queue_pos.x.u8(), queue_pos.y.u8());

                    // -1 for room exits.
                    if dx == 0 || dy == 0 || dx >= self.width - 1 || dy >= self.height - 1 {
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

        let mut cdm = CompressedDirectionMatrix::new();

        for x in 0..self.width {
            for y in 0..self.height {
                let index = (y as u16 * self.width as u16 + x as u16) as usize;
                cdm.set_xy(x, y, self.directions.as_ref().unwrap()[index]);
            }
        }

        cdm
    }
}

pub fn visualise_field(room: &Room, field: &CompressedDirectionMatrix) {
    for x in 0..50 {
        for y in 0..50 {
            let dir = field.get_xy(x, y);
            if dir == 0 {
                continue;
            }

            let from_pos = Position::new(RoomCoordinate::new(x).unwrap(), RoomCoordinate::new(y).unwrap(), room.name());
            let vis = room.visual();

            //info!("Drawing line from {:?} to {:?}", from_pos, num_to_dir(dir));
            let dirt = num_to_dir(dir);

            vis.text(from_pos.x().u8() as f32, from_pos.y().u8() as f32, dirt.to_string(), Some(TextStyle::default().align(screeps::TextAlign::Center)));
        }
    }
}