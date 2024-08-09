use screeps::Direction;

use crate::{constants::ROOM_AREA, movement::movement_utils::num_to_dir};

#[derive(Debug, Clone)]
pub struct CompressedMatrix {
    pub matrix: [u8; ROOM_AREA / 2]
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CompressedMatrix {
    pub fn new() -> CompressedMatrix {
        CompressedMatrix {
            matrix: [0; ROOM_AREA / 2]
        }
    }

    pub fn get_xy(&self, x: u8, y: u8) -> u8 {
        let index = y as u16 * 50 + x as u16;
        if index & 1 == 0 {
            self.matrix[index as usize / 2] & 0b00001111
        } else {
            self.matrix[index as usize / 2] >> 4
        }
    }

    pub fn get_dir(&self, x: u8, y: u8) -> Option<Direction> {
        let dir = self.get_xy(x, y);

        if dir == 0 || dir > 8 {
            None
        } else {
            Some(num_to_dir(dir))
        }
    }

    pub fn set_xy(&mut self, x: u8, y: u8, value: u8) {
        let index = y as u16 * 50 + x as u16;

        let value = if value > 15 {
            0
        } else {
            value
        };

        if index & 1 == 0 {
            let previous_other_half = self.matrix[index as usize / 2] >> 4;

            self.matrix[index as usize / 2] = (previous_other_half << 4) | value;
        } else {
            let previous_other_half = self.matrix[index as usize / 2] & 0b00001111;

            self.matrix[index as usize / 2] = (value << 4) | previous_other_half;
        }
    }
}
