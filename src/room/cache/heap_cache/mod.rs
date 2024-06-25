use std::collections::HashMap;

use hauling::HeapHaulingCache;
use screeps::{Creep, ObjectId, Room, Source};

use crate::traits::room::RoomExtensions;

pub mod hauling;

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
    pub hauling: HeapHaulingCache,

    pub sources: Vec<ObjectId<Source>>,
}

#[derive(Debug, Clone)]
pub struct HeapCreep {
    pub health: u32,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomHeapCache {
    pub fn new(room: &Room) -> RoomHeapCache {
        RoomHeapCache {
            room: room.name_str(),
            creeps: HashMap::new(),
            hauling: HeapHaulingCache::default(),

            sources: Vec::new(),
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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