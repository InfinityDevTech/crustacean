use std::collections::HashMap;
use rand::{prelude::SliceRandom, rngs::StdRng, Rng, SeedableRng};

use screeps::{game, Creep, HasId, HasPosition, Position, RawObjectId, ResourceType, SharedCreepProperties};
use serde::{Deserialize, Serialize};

use crate::{memory::{CreepHaulTask, ScreepsMemory}, utils::scale_haul_priority};

use super::structures::RoomStructureCache;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HaulingPriority {
    Combat = 70,
    Emergency = 60,
    Spawning = 50,
    Ruins = 40,
    Energy = 30,
    Minerals = 20,
    Market = 10,
    Storage = 0,
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
    pub resource: Option<ResourceType>,
    pub amount: Option<u32>,
    pub priority: f32,
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

    pub fn create_order(&mut self, target: RawObjectId, resource: Option<ResourceType>, amount: Option<u32>, priority: f32, haul_type: HaulingType) {
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
            orders.retain(|rsc| rsc.resource == Some(resource_type));
        }

        orders.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        let mut seedable = StdRng::seed_from_u64(game::time().into());

        if let Some(order) = orders.clone().into_iter().next() {
            let id = order.id;
            orders.retain(|o| o.priority == order.priority);

            let order_int = &mut seedable.gen_range(0..orders.len());
            let order = orders.get(*order_int).unwrap();

            let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
            let task = CreepHaulTask {
                target_id: order.target,
                resource: order.resource.unwrap_or(ResourceType::Energy),
                amount: order.amount,
                priority: order.priority,
                haul_type: order.haul_type,
            };


            if let Some(order_amount) = order.amount {
                let creep_carry_capacity = creep.store().get_free_capacity(Some(task.resource));

                if order_amount as i32 - creep_carry_capacity < 0 {
                    self.new_orders.remove(&id);
                } else {
                    self.new_orders.get_mut(&id).unwrap().amount = Some((order_amount as i32 - creep_carry_capacity) as u32);
                }
            }

            creep_memory.hauling_task = Some(task);
            return creep_memory.hauling_task.clone();
        }
        None
    }

    pub fn haul_storage(&mut self, structures: &RoomStructureCache) {
        let storage = &structures.storage;

        if let Some(storage) = storage {
            if storage.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                self.create_order(
                    storage.raw_id(),
                    Some(ResourceType::Energy),
                    Some(storage.store().get_used_capacity(Some(ResourceType::Energy))),
                    0.0,
                    HaulingType::Offer
                )
            }

            if storage.store().get_free_capacity(None) > 0 {
                self.create_order(
                    storage.raw_id(),
                    None,
                    Some(storage.store().get_free_capacity(None).try_into().unwrap()),
                    0.0,
                    HaulingType::Transfer
                )
            }
        }
    }

    pub fn haul_ruins(&mut self, structures: &RoomStructureCache) {
        let ruins = &structures.ruins;

        for ruin in ruins.values() {
            let energy_amount = ruin.store().get_used_capacity(Some(ResourceType::Energy));

            if energy_amount > 0 {
                self.create_order(
                    ruin.raw_id(),
                    Some(ResourceType::Energy),
                    Some(ruin.store().get_used_capacity(Some(ResourceType::Energy))),
                    scale_haul_priority(ruin.store().get_capacity(None), energy_amount, HaulingPriority::Ruins, false),
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