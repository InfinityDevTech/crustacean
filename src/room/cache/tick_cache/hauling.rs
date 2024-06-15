use std::{collections::HashMap, ops::Div};

use log::info;
use rust_decimal::Decimal;
use screeps::{game, Creep, HasId, HasPosition, Position, RawObjectId, ResourceType, SharedCreepProperties, TextStyle};
use serde::{Deserialize, Serialize};

use crate::{heap, heap_cache, memory::{CreepHaulTask, Role, ScreepsMemory}, room::cache::heap_cache::RoomHeapCache, utils::scale_haul_priority};

use super::CachedRoom;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HaulingPriority {
    Combat = 0,
    Emergency = 9,
    FastFillerContainer = 2,
    Spawning = 5,
    Ruins = 30,
    Energy = 40,
    Upgrading = 100,
    Minerals = 90,
    Market = 101,
    Storage = 110,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HaulingCache {
    pub new_orders: HashMap<u32, RoomHaulingOrder>,

    pub current_id_index: u32,

    pub haulers: Vec<String>,

    iterator_salt: u32,
}

impl HaulingCache {
    pub fn new() -> HaulingCache {
        HaulingCache {
            new_orders: HashMap::new(),
            current_id_index: game::time(),
            haulers: Vec::new(),

            iterator_salt: 0,
        }
    }

    pub fn get_unique_id(&mut self) -> u32 {
        self.current_id_index += 1;
        self.current_id_index
    }

    pub fn create_order(&mut self, target: RawObjectId, resource: Option<ResourceType>, amount: Option<u32>, priority: f32, haul_type: HaulingType) {
        let id = self.get_unique_id();

        if game::get_object_by_id_erased(&target).unwrap().pos().room_name() != "E12N6" {
            info!("MAklking req {}", game::get_object_by_id_erased(&target).unwrap().pos().room_name());
        }

        let decimal = Decimal::new(priority.round() as i64, 0).div(Decimal::new(200, 1));

        let order = RoomHaulingOrder {
            id,
            target,
            resource,
            amount,
            priority: decimal.round_dp(2).try_into().unwrap(),
            haul_type,
        };

        self.new_orders.insert(id, order);
    }

    pub fn find_new_order(&mut self, creep: &Creep, memory: &mut ScreepsMemory, resource: Option<ResourceType>, order_type: Vec<HaulingType>, heap_cache: &mut RoomHeapCache) -> Option<CreepHaulTask> {
        //let pre_find_cpu = game::cpu::get_used();
        let mut orders = self.new_orders.values().collect::<Vec<&RoomHaulingOrder>>().clone();

        orders.retain(|x| order_type.contains(&x.haul_type));

        let mut top_scorer: Option<RoomHaulingOrder> = None;
        let mut current_score: f32 = f32::MAX;

        for order in orders {
            if let Some(resource_type) = resource {
                if order.resource != Some(resource_type) { continue; }
            }

            if creep.pos().room_name() != game::get_object_by_id_erased(&order.target).unwrap().pos().room_name() { info!("Romm dont match") }

            let structure = game::get_object_by_id_erased(&order.target);
            if structure.is_none() { continue; }

            let distance_to_target = structure.as_ref().unwrap().pos().get_range_to(creep.pos());
            let priority = order.priority;

            let score = (distance_to_target as f32 * 0.75) + priority;

            let vis = structure.as_ref().unwrap().room().unwrap().visual();

            let x = structure.as_ref().unwrap().pos().x().u8() as f32;
            let y = structure.as_ref().unwrap().pos().y().u8() as f32;
            vis.text(
                x,
                y,
                format!("{:?}", score),
                Some(TextStyle::default().font(0.5).align(screeps::TextAlign::Center)),
            );

            if order.haul_type == HaulingType::Transfer {
                vis.text(
                    x,
                    y - 1.0,
                    "T".to_string(),
                    Some(TextStyle::default().font(0.5)),
                );
            } else {
                vis.text(
                    x,
                    y - 1.0,
                    "O".to_string(),
                    Some(TextStyle::default().font(0.5)),
                );
            }

            let reserved = if let Some(reserved_order) = heap_cache.hauling.reserved_orders.get(&order.target) {
                let creep = game::creeps().get(reserved_order.to_string());
                let mut reserved = true;

                if creep.is_none() {
                    heap_cache.hauling.reserved_orders.remove(&order.target);

                    reserved = false;
                }

                reserved
            } else {
                false
            };

            if score >= current_score || reserved { continue; }

            //info!("Current_score {}", score);

            current_score = score;
            top_scorer = Some(order.clone());
        }

        top_scorer.as_ref()?;
        let order = top_scorer.unwrap();

        let task = CreepHaulTask {
            target_id: order.target,
            resource: order.resource.unwrap_or(ResourceType::Energy),
            amount: order.amount,
            priority: order.priority,
            haul_type: order.haul_type
        };

        self.new_orders.remove(&order.id);

        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

        let storage_amount = creep.store().get_free_capacity(Some(task.resource));

        // Only reserve if the order amount isnt specified
        // or if the order amount is less than the free capacity of the creep
        if order.haul_type == HaulingType::Offer || order.haul_type == HaulingType::Pickup || order.haul_type == HaulingType::Withdraw {
            if let Some(order_amount) = order.amount {
                if order_amount as i32 - storage_amount < 0 {
                    heap_cache.hauling.reserved_orders.insert(order.target, creep.name().to_string());
                }
            }
        } else {
            heap_cache.hauling.reserved_orders.insert(order.target, creep.name().to_string());
        }

        creep_memory.hauling_task = Some(task);
        creep_memory.hauling_task.clone()
    }

    /*
    orders.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

            let mut seedable = StdRng::seed_from_u64((game::time() + self.iterator_salt).into());
            self.iterator_salt += 1;

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
            None */
}

impl CreepHaulTask {
    pub fn get_target_position(&self) -> Option<Position> {
        let target = game::get_object_by_id_erased(&self.target_id);

        target.as_ref()?;

        Some(target.unwrap().pos())
    }
}

pub fn haul_spawn(room_cache: &mut CachedRoom) {
    let has_ff = !room_cache.creeps.creeps_of_role.get(&Role::FastFiller).unwrap_or(&Vec::new()).is_empty();

    for spawn in room_cache.structures.spawns.values() {
        if spawn.store().get_free_capacity(Some(ResourceType::Energy)) == 0 || (has_ff && room_cache.structures.containers.fast_filler.is_some()) { continue }

        let priority = scale_haul_priority(
            spawn.store().get_capacity(Some(ResourceType::Energy)),
            spawn.store().get_free_capacity(Some(ResourceType::Energy)) as u32,
            HaulingPriority::Spawning,
            true
        );

        room_cache.hauling.create_order(
            spawn.raw_id(),
            Some(ResourceType::Energy),
            Some(spawn.store().get_free_capacity(Some(ResourceType::Energy)) as u32),
            priority,
            HaulingType::Transfer
        );
    }
}

pub fn haul_ruins(room_cache: &mut CachedRoom) {
    let ruins = &room_cache.structures.ruins;

    for ruin in ruins.values() {
        let energy_amount = ruin.store().get_used_capacity(Some(ResourceType::Energy));

        if energy_amount > 0 {
            room_cache.hauling.create_order(
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

pub fn haul_storage(room_cache: &mut CachedRoom) {
    let storage = &room_cache.structures.storage;

    if let Some(storage) = storage {
        if storage.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
            room_cache.hauling.create_order(
                storage.raw_id(),
                Some(ResourceType::Energy),
                Some(storage.store().get_used_capacity(Some(ResourceType::Energy))),
                f32::MAX - 100.0,
                HaulingType::Offer
            )
        }

        if storage.store().get_free_capacity(None) > 0 {
            room_cache.hauling.create_order(
                storage.raw_id(),
                None,
                Some(storage.store().get_free_capacity(None).try_into().unwrap()),
                f32::MAX - 100.0,
                HaulingType::Transfer
            )
        }
    }
}

pub fn haul_extensions(room_cache: &mut CachedRoom) {
    for source in room_cache.structures.extensions.values() {
        if source.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
            let priority = scale_haul_priority(
                source.store().get_capacity(Some(ResourceType::Energy)),
                source.store().get_used_capacity(Some(ResourceType::Energy)),
                HaulingPriority::Spawning,
                true
            );

            room_cache.hauling.create_order(
                source.raw_id(),
                Some(ResourceType::Energy),
                Some(source
                    .store()
                    .get_free_capacity(Some(ResourceType::Energy))
                    .try_into()
                    .unwrap()),
                priority,
                HaulingType::Transfer,
            );
        }
    }
}