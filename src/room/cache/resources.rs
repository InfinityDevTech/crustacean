use std::collections::HashMap;

use screeps::{find, HasId, Resource, ResourceType, Room};

use crate::memory::ScreepsMemory;

use super::hauling::{HaulingCache, HaulingPriority, HaulingType};

#[derive(Debug, Clone)]
pub struct RoomResourceCache {
    pub dropped_energy: Vec<Resource>,

    pub dropped_resources: HashMap<ResourceType, Vec<Resource>>,
}

impl RoomResourceCache {
    pub fn new_from_room(room: &Room, _memory: &mut ScreepsMemory) -> RoomResourceCache {
        let mut cache = RoomResourceCache {
            dropped_energy: Vec::new(),

            dropped_resources: HashMap::new(),
        };

        cache.refresh_resource_cache(room);
        cache
    }

    pub fn create_haul_request_for_dropped_energy(&self, hauling: &mut HaulingCache) {
        for resource in &self.dropped_energy {
            hauling.create_order(resource.id().into(), resource.resource_type(), resource.amount(), HaulingPriority::Energy, HaulingType::Pickup);
        }
    }

    pub fn refresh_resource_cache(&mut self, room: &Room) {
        let dropped_resources = room.find(find::DROPPED_RESOURCES, None);

        for resource in dropped_resources {
            if resource.resource_type() == screeps::ResourceType::Energy {
                self.dropped_energy.push(resource);
            } else if let Some(resource_vec) = self.dropped_resources.get_mut(&resource.resource_type()) {
                resource_vec.push(resource);
            } else {
                self.dropped_resources.insert(resource.resource_type(), vec![resource]);
            }
        }
    }
}