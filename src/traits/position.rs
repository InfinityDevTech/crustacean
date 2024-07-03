use screeps::{game, Position, RoomName, RoomXY, Terrain};

pub trait PositionExtensions {
    fn get_accessible_positions_around(&self, range: u8) -> u8;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl PositionExtensions for Position {
    fn get_accessible_positions_around(&self, range: u8) -> u8 {
        let mut positions = 0;
        let terrain = game::map::get_room_terrain(self.room_name()).unwrap();

        for x in (self.x().u8() as i8 - range as i8)..=(self.x().u8() as i8 + range as i8) {
            for y in (self.y().u8() as i8 - range as i8)..=(self.y().u8() as i8 + range as i8) {
                let terrain_type = terrain.get(x as u8, y as u8);

                if terrain_type == Terrain::Plain || terrain_type == Terrain::Swamp {
                    positions += 1;
                }
            }
        }
        positions
    }
}

pub trait RoomXYExtensions {
    fn as_position(&self, room_name: &RoomName) -> Position;
}

impl RoomXYExtensions for RoomXY {
    fn as_position(&self, room_name: &RoomName) -> Position {
        Position::new(self.x, self.y, *room_name)
    }
}