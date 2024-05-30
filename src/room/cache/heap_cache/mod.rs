use screeps::{Creep, Room};

use crate::traits::room::RoomExtensions;

pub enum HealthChangeType {
    Damage,
    Heal,
    None,
}

pub struct RoomHeapCache {
    pub room: String,
    pub creeps: Vec<HeapCreep>,
}

pub struct HeapCreep {
    pub health: u32,
}

impl RoomHeapCache {
    pub fn new(room: &Room) -> RoomHeapCache {
        RoomHeapCache {
            room: room.name_str(),
            creeps: Vec::new(),
        }
    }
}

impl HeapCreep {
    pub fn new(health: u32) -> HeapCreep {
        HeapCreep {
            health,
        }
    }

    pub fn get_health_change(&mut self, creep: &Creep) -> HealthChangeType {
        let change_type = match creep.hits() {
            h if h < self.health => HealthChangeType::Damage,
            h if h > self.health => HealthChangeType::Heal,
            _ => HealthChangeType::None,
        };

        self.health = creep.hits();

        change_type
    }
}