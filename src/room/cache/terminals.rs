use std::collections::HashMap;

use screeps::{ResourceType, RoomName};

pub struct TerminalCache {
    pub needs_by_resource: HashMap<ResourceType, Vec<RoomName>>,
    pub needs_by_room: HashMap<RoomName, Vec<ResourceType>>,

    pub offers_by_resource: HashMap<ResourceType, Vec<RoomName>>,
    pub offers_by_room: HashMap<RoomName, Vec<ResourceType>>,
}

impl TerminalCache {
    pub fn new() -> TerminalCache {
        TerminalCache {
            needs_by_resource: HashMap::new(),
            needs_by_room: HashMap::new(),

            offers_by_resource: HashMap::new(),
            offers_by_room: HashMap::new(),
        }
    }

    pub fn mark_offer_resource(&mut self, room: RoomName, resource: Vec<ResourceType>) {
        if let Some(offers) = self.offers_by_room.get_mut(&room) {
            for resource in resource.clone() {
                if !offers.contains(&resource) {
                    offers.push(resource);
                }
            }
        } else {
            self.offers_by_room.insert(room, resource.clone());
        }

        for resource in &resource {
            if let Some(offers) = self.offers_by_resource.get_mut(resource) {
                if !offers.contains(&room) {
                    offers.push(room);
                }
            } else {
                self.offers_by_resource.insert(*resource, vec![room]);
            }
        }
    }

    pub fn mark_needs_resource(&mut self, room: RoomName, resource: Vec<ResourceType>) {
        if let Some(needs) = self.needs_by_room.get_mut(&room) {
            for resource in resource.clone() {
                if !needs.contains(&resource) {
                    needs.push(resource);
                }
            }
        } else {
            self.needs_by_room.insert(room, resource.clone());
        }

        for resource in &resource {
            if let Some(needs) = self.needs_by_resource.get_mut(resource) {
                if !needs.contains(&room) {
                    needs.push(room);
                }
            } else {
                self.needs_by_resource.insert(*resource, vec![room]);
            }
        }
    }
}