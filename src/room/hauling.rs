use screeps::{game, Creep, HasId, HasPosition, ObjectId, Position, RawObjectId, ResourceType, SharedCreepProperties, Structure};
use serde::{Deserialize, Serialize};

use crate::memory::{CreepMemory, HaulOrder, RoomMemory, ScreepsMemory};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HaulPriorities {
    Combat = 0,
    Spawning = 1,
    Defensive = 2,
    Normal = 3,
    Storage = 4,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HaulType {
    Pickup = 0,
    Withdraw = 1,
    Deposit = 2,
}

impl HaulOrder {
    pub fn add_responder(&mut self, creep: &Creep) {
        self.responder = Some(creep.name());
    }

    pub fn get_target_position(&self) -> Position {
        let target = game::get_object_by_id_erased(&self.target_id).unwrap();

        target.pos()
    }
}

impl RoomMemory {
    pub fn create_unique_id(&mut self) -> u128 {
        let id = self.id;
        self.id += 1;
        id
    }
    pub fn destroy_haul_order(&mut self, order_id: u128) {
        self.haul_orders.remove(&order_id);
    }

    pub fn create_haul_order(&mut self, priority: HaulPriorities, target_id: RawObjectId, resource: ResourceType, amount: u32, haul_type: HaulType) {
        let t = self.clone();
        let mut existing_orders = t.haul_orders.values().filter(|order| order.target_id == target_id && order.haul_type == haul_type);

        if existing_orders.clone().count() > 0 {
            let order = existing_orders.next().unwrap();

            let new_amount = order.amount + amount;
            self.haul_orders.get_mut(&order.id).unwrap().amount = new_amount;
        }
        let id = self.create_unique_id();

        let order = HaulOrder {
            id,
            priority,
            target_id,
            target_type: resource,
            responder: None,
            haul_type,
            amount,
        };

        self.haul_orders.insert(id, order);
    }

    pub fn get_haul_order(&self, order_id: u128) -> Option<&HaulOrder> {
        self.haul_orders.get(&order_id)
    }

    pub fn get_haul_order_mut(&mut self, order_id: u128) -> Option<&mut HaulOrder> {
        self.haul_orders.get_mut(&order_id)
    }
}