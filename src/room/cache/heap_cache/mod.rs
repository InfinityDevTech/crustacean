use std::collections::HashMap;

use log::info;
use screeps::{Creep, HasPosition, ObjectId, Position, Room, SharedCreepProperties, Source};

use crate::traits::room::RoomExtensions;

pub mod hauling;

// TODO:
// Fix this shit, why do I have two different types of heap
// Something tells me that they arent saving properly, but I know the other heap does
// So, im just going to move a "Heap Creep", to the heap and see what happens
// Also, stuck detection is broken because of this shit.

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

    pub sources: Vec<ObjectId<Source>>,
}

#[derive(Debug, Clone)]
pub struct HeapCreep {
    pub health: u32,
    pub previous_position: Position,
    pub stuck_time: u8,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomHeapCache {
    pub fn new(room: &Room) -> RoomHeapCache {
        RoomHeapCache {
            room: room.name_str(),
            creeps: HashMap::new(),

            sources: Vec::new(),
        }
    }
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl HeapCreep {
    pub fn new(creep: &Creep) -> HeapCreep {
        HeapCreep {
            health: creep.hits(),
            previous_position: creep.pos(),
            stuck_time: 0,
        }
    }

    pub fn update_position(&mut self, creep: &Creep) {
        if creep.pos() == self.previous_position {
            self.stuck_time += 1;
        } else {
            self.stuck_time = 0;
        }

        self.previous_position = creep.pos();
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