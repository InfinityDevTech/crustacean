use screeps::{CostMatrix, LocalCostMatrix, Room};

use crate::{room::cache::RoomCache, traits::room::RoomExtensions};

use super::cost_matrix;

pub struct RoomTraffic {
    pub room_cost_matrix: LocalCostMatrix
}

impl RoomTraffic {
    pub fn new(room: &Room, cache: &mut RoomCache) -> RoomTraffic {

        let room_cost_matrix = if room.my() {
            cost_matrix::owned_room(room, cache)
        } else {
            let _ = "test";
            cost_matrix::owned_room(room, cache)
        };

        RoomTraffic {
            room_cost_matrix
        }
    }
}