use std::collections::HashMap;

use screeps::RoomName;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomReservationGoal {
    pub reservation_target: RoomName,
    pub accessible_reservation_spots: u8,
    pub creeps_assigned: Vec<String>,
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone, Default)]]
    pub struct GoalMemory {
        pub room_reservation: HashMap<RoomName, RoomReservationGoal>,
    }
}