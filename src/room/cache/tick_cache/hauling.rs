use std::collections::HashMap;

use screeps::{game, Creep, HasId, HasPosition, Position, RawObjectId, ResourceType, SharedCreepProperties};
use serde::{Deserialize, Serialize};

use crate::memory::{CreepHaulTask, ScreepsMemory};

use super::structures::RoomStructureCache;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HaulingPriority {
    Combat = 0,
    Emergency = 1,
    Spawning = 2,
    Energy = 3,
    Minerals = 4,
    Market = 5
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HaulingType {
    Offer = 0,
    Withdraw = 1,
    Pickup = 2,
    Transfer = 3
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomHaulingOrder {
    pub id: u32,
    pub target: RawObjectId,
    pub resource: ResourceType,
    pub amount: u32,
    pub priority: HaulingPriority,
    pub haul_type: HaulingType,
}

pub struct HaulingCache {
    pub new_orders: HashMap<u32, RoomHaulingOrder>,

    pub current_id_index: u32,

    pub haulers: Vec<String>,
}

impl HaulingCache {
    pub fn new() -> HaulingCache {
        HaulingCache {
            new_orders: HashMap::new(),
            current_id_index: game::time(),
            haulers: Vec::new(),
        }
    }

    pub fn get_unique_id(&mut self) -> u32 {
        self.current_id_index += 1;
        self.current_id_index
    }

    pub fn create_order(&mut self, target: RawObjectId, resource: ResourceType, amount: u32, priority: HaulingPriority, haul_type: HaulingType) {
        let id = self.get_unique_id();

        let order = RoomHaulingOrder {
            id,
            target,
            resource,
            amount,
            priority,
            haul_type,
        };

        self.new_orders.insert(id, order);
    }

    pub fn find_new_order(&mut self, creep: &Creep, memory: &mut ScreepsMemory, resource: Option<ResourceType>, order_type: Vec<HaulingType>) -> Option<CreepHaulTask> {
        let mut orders = self.new_orders.values().collect::<Vec<&RoomHaulingOrder>>().clone();

        orders.retain(|x| order_type.contains(&x.haul_type));

        if let Some(resource_type) = resource {
            orders.retain(|rsc| rsc.resource == resource_type);
        }

        orders.sort_by(|a, b| a.priority.cmp(&b.priority));

        if let Some(order) = orders.into_iter().next() {
            let id = order.id;
            let order = self.new_orders.get_mut(&id).unwrap();

            let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
            let task = CreepHaulTask {
                target_id: order.target,
                resource: order.resource,
                amount: order.amount,
                priority: order.priority,
                haul_type: order.haul_type,
            };

            let creep_carry_capacity = creep.store().get_free_capacity(Some(ResourceType::Energy));
            let order_amount = order.amount as i32;

            if order_amount > creep_carry_capacity {
                order.amount = (order_amount - creep_carry_capacity) as u32;
            } else {
                self.new_orders.remove(&id);
            }
            
            creep_memory.hauling_task = Some(task);
            return creep_memory.hauling_task.clone();
        }
        None
    }

    pub fn haul_ruins(&mut self, structures: &RoomStructureCache) {
        let ruins = &structures.ruins;

        for ruin in ruins.values() {
            if ruin.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                self.create_order(
                    ruin.raw_id(),
                    ResourceType::Energy,
                    ruin.store().get_used_capacity(Some(ResourceType::Energy)),
                    HaulingPriority::Energy,
                    HaulingType::Offer
                );
                return;
            }
        }
    }
}

impl CreepHaulTask {
    pub fn get_target_position(&self) -> Option<Position> {
        let target = game::get_object_by_id_erased(&self.target_id);

        target.as_ref()?;

        Some(target.unwrap().pos())
    }
}