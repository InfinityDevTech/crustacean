pub trait RoomExtensions {
    fn name_str(&self) -> String;
}

impl RoomExtensions for screeps::Room {
    fn name_str(&self) -> String {
        self.name().to_string()
    }
}