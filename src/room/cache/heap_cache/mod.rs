use std::collections::HashMap;

use screeps::{Creep, Room};

use crate::traits::room::RoomExtensions;

#[derive(PartialEq)]
pub enum HealthChangeType {
    Damage,
    Heal,
    None,
}

#[derive(Debug, Clone)]
pub struct RoomHeapCache {
    pub room: String,
    pub creeps: HashMap<String, HeapCreep>,
}

#[derive(Debug, Clone)]
pub struct HeapCreep {
    pub health: u32,
}

impl RoomHeapCache {
    pub fn new(room: &Room) -> RoomHeapCache {
        RoomHeapCache {
            room: room.name_str(),
            creeps: HashMap::new(),
        }
    }
}

impl HeapCreep {
    pub fn new(creep: &Creep) -> HeapCreep {
        HeapCreep {
            health: creep.hits(),
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