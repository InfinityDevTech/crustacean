use std::collections::HashMap;

use screeps::{Creep, ObjectId, RoomXY};

#[derive(Debug, Clone)]
pub struct TrafficCache {
    pub matched_coord: HashMap<ObjectId<Creep>, RoomXY>,
    pub intended_move: HashMap<ObjectId<Creep>, RoomXY>,

    pub movement_map: HashMap<RoomXY, ObjectId<Creep>>,

    pub cached_ops: HashMap<ObjectId<Creep>, Vec<RoomXY>>,
    pub move_intents: u8,
}

impl TrafficCache {
    pub fn new() -> Self {
        Self {
            matched_coord: HashMap::new(),
            intended_move: HashMap::new(),
            movement_map: HashMap::new(),
            cached_ops: HashMap::new(),
            move_intents: 0,
        }
    }
}