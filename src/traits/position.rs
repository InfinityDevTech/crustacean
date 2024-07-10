use screeps::{game, Position, RoomCoordinate, RoomName, RoomXY, Terrain};

pub trait PositionExtensions {
    fn get_accessible_positions_around(&self, range: u8) -> Vec<Position>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl PositionExtensions for Position {
    fn get_accessible_positions_around(&self, range: u8) -> Vec<Position> {
        let mut positions = Vec::new();
        let terrain = game::map::get_room_terrain(self.room_name()).unwrap();

        for x in (self.x().u8() as i8 - range as i8)..=(self.x().u8() as i8 + range as i8) {
            for y in (self.y().u8() as i8 - range as i8)..=(self.y().u8() as i8 + range as i8) {
                let terrain_type = terrain.get(x as u8, y as u8);

                let x = unsafe { RoomCoordinate::unchecked_new(x as u8) };
                let y = unsafe { RoomCoordinate::unchecked_new(y as u8) };

                if terrain_type == Terrain::Plain || terrain_type == Terrain::Swamp {
                    positions.push(Position::new(x, y, self.room_name()));
                }
            }
        }
        positions
    }
}

pub trait RoomXYExtensions {
    fn as_position(&self, room_name: &RoomName) -> Position;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomXYExtensions for RoomXY {
    fn as_position(&self, room_name: &RoomName) -> Position {
        Position::new(self.x, self.y, *room_name)
    }
}