use screeps::{Creep, HasId, ResourceType, SharedCreepProperties, Structure};
use serde::{Deserialize, Serialize};

use crate::memory::{HaulOrder, RoomMemory};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HaulPriorities {
    Combat = 0,
    Spawning = 1,
    Defensive = 2,
    Normal = 3,
}

impl HaulOrder {
    pub fn add_responder(&mut self, creep: &Creep) {
        self.responder = Some(creep.name());
    }
}

impl RoomMemory {
    pub fn create_unique_id(&mut self) -> u128 {
        let id = self.id;
        self.id += 1;
        id
    }
    pub fn destroy_haul_order(&mut self, order_id: u128) {
        self.haul_orders.retain(|x| x.id != order_id);
    }

    pub fn create_haul_order(&mut self, priority: HaulPriorities, target: Structure, resource: ResourceType, amount: u32) {
        let id = self.create_unique_id();

        let order = HaulOrder {
            id,
            priority,
            target_id: target.id(),
            target_type: resource,
            responder: None,
            amount,
        };

        self.haul_orders.push(order);
    }
}