use std::{collections::HashMap, ops::Div};

use log::{info, warn};
use rust_decimal::Decimal;
use screeps::{
    creep, game, CircleStyle, Creep, HasId, HasPosition, Position, RawObjectId, ResourceType, RoomName, RoomVisual, SharedCreepProperties, StructureProperties, StructureType, TextStyle
};
use serde::{Deserialize, Serialize};

use crate::{
    heap, heap_cache,
    memory::{CreepHaulTask, Role, ScreepsMemory},
    room::{
        cache::heap_cache::{
            hauling::{self, HeapHaulingReservation},
            RoomHeapCache,
        },
        creeps::local::{base_hauler, hauler::execute_order},
    },
    utils::{name_to_role, scale_haul_priority},
};

use super::{CachedRoom, RoomCache};

// Priorities are 1:1 now.
// No more fucking decimal scaling. Im amazed marvin can do it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HaulingPriority {
    Combat = 0,
    Emergency = 20,
    FastFillerContainer = 70,
    Spawning = 5,
    Ruins = 30,
    Energy = 40,
    Upgrading = 85,
    Minerals = 90,
    Market = 102,
    Storage = 110,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HaulingType {
    Offer = 0,
    Withdraw = 1,
    Pickup = 2,
    Transfer = 3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomHaulingOrder {
    pub id: u32,
    pub target: RawObjectId,
    pub target_type: Option<StructureType>,
    pub resource: Option<ResourceType>,
    pub amount: Option<u32>,
    pub priority: f32,
    pub haul_type: HaulingType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HaulTaskRequest {
    pub creep_name: String,

    pub haul_type: Vec<HaulingType>,
    pub resource_type: Option<ResourceType>,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl HaulTaskRequest {
    pub fn creep_name(&mut self, creep_name: String) -> &mut Self {
        self.creep_name = creep_name;
        self
    }

    pub fn haul_type(&mut self, haul_type: Vec<HaulingType>) -> &mut Self {
        self.haul_type = haul_type;
        self
    }

    pub fn resource_type(&mut self, resource_type: ResourceType) -> &mut Self {
        self.resource_type = Some(resource_type);
        self
    }

    pub fn finish(&mut self) -> HaulTaskRequest {
        self.clone()
    }
}

impl Default for HaulTaskRequest {
    fn default() -> Self {
        HaulTaskRequest {
            creep_name: String::new(),

            haul_type: vec![
                HaulingType::Withdraw,
                HaulingType::Pickup,
                HaulingType::Transfer,
            ],
            resource_type: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HaulingCache {
    pub new_orders: HashMap<u32, RoomHaulingOrder>,
    pub current_id_index: u32,
    pub haulers: Vec<String>,
    pub reserved_order_distances: HashMap<String, u32>,

    pub wanting_orders: Vec<HaulTaskRequest>,

    creeps_matched: HashMap<String, bool>,
    orders_matched: HashMap<u32, bool>,

    orders_matched_to_creeps: HashMap<u32, String>,
    creeps_matched_to_orders: HashMap<String, RoomHaulingOrder>,

    order_score_matches: HashMap<u32, u32>,
    creep_score_matches: HashMap<String, f32>,

    iterator_salt: u32,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl HaulingCache {
    pub fn new() -> HaulingCache {
        HaulingCache {
            new_orders: HashMap::new(),
            current_id_index: game::time(),
            haulers: Vec::new(),

            wanting_orders: Vec::new(),

            creeps_matched: HashMap::new(),
            orders_matched: HashMap::new(),
            orders_matched_to_creeps: HashMap::new(),
            creeps_matched_to_orders: HashMap::new(),
            order_score_matches: HashMap::new(),
            creep_score_matches: HashMap::new(),

            reserved_order_distances: HashMap::new(),
            iterator_salt: 0,
        }
    }

    pub fn get_unique_id(&mut self) -> u32 {
        self.current_id_index += 1;
        self.current_id_index
    }

    pub fn create_order(
        &mut self,
        target: RawObjectId,
        target_type: Option<StructureType>,
        resource: Option<ResourceType>,
        amount: Option<u32>,
        priority: f32,
        haul_type: HaulingType,
    ) {
        let id = self.get_unique_id();

        let decimal = Decimal::new(priority.round() as i64, 0).div(Decimal::new(200, 1));

        let mut order = RoomHaulingOrder {
            id,
            target,
            target_type,
            resource,
            amount,
            //priority: decimal.round_dp(2).try_into().unwrap(),
            priority: priority.round(),
            haul_type,
        };

        if let Some(target_pos) = &order.get_target_position() {
            let mut lock = heap().rooms.lock().unwrap();
            if let Some(heap_cache) = lock.get_mut(&target_pos.room_name()) {
                if let Some(existing_order) = heap_cache.hauling.reserved_orders.get(&order.target)
                {
                    let reserved = existing_order.amount_reserved;

                    if game::time() % 10 == 0 {
                        for creep in existing_order.creeps_assigned.iter() {
                            if game::creeps().get(creep.to_string()).is_none() {
                                continue;
                            }
                        }
                    }

                    if order.amount > Some(reserved) {
                        order.amount = Some(order.amount.unwrap() - reserved);
                    } else {
                        return;
                    }
                }
            }
        }

        let position = game::get_object_by_id_erased(&order.target).unwrap().pos();
        let room_visual = game::rooms().get(position.room_name()).unwrap().visual();

        room_visual.circle(
            position.x().u8() as f32,
            position.y().u8() as f32,
            Some(
                CircleStyle::default()
                    .radius(0.25)
                    .stroke("#ff0000")
                    .fill("#ff0000")
                    .opacity(0.5),
            ),
        );

        if order.haul_type == HaulingType::Transfer {
            room_visual.text(
                position.x().u8() as f32,
                position.y().u8() as f32,
                format!("T {:.2}", order.priority),
                Some(TextStyle::default().color("#ff0000")),
            );
        } else {
            room_visual.text(
                position.x().u8() as f32,
                position.y().u8() as f32,
                format!("P {:.2}", order.priority),
                Some(TextStyle::default().color("#00ff00")),
            );
        }

        self.new_orders.insert(id, order);
    }
}

pub fn match_haulers(cache: &mut RoomCache, memory: &mut ScreepsMemory, room_name: &RoomName) {
    let starting_cpu = game::cpu::get_used();
    let mut matched_creeps = Vec::new();

    let hauling_cache = &mut cache.rooms.get_mut(room_name).unwrap().hauling;

    if hauling_cache.wanting_orders.is_empty() {
        info!("  [HAULING] No haulers wanting orders");
        return;
    }

    for hauler in hauling_cache.wanting_orders.iter() {
        let game_creep = game::creeps().get(hauler.creep_name.to_string()).unwrap();

        let mut top_scorer = None;
        let mut top_score = f32::MAX;

        let role = name_to_role(&hauler.creep_name);

        for order in hauling_cache.new_orders.values() {
            if order.target_type == Some(StructureType::Storage) && hauler.haul_type.contains(&HaulingType::Offer) && role == Some(Role::Hauler) {
                continue;
            }

            if let Some(resource_type) = hauler.resource_type {
                if order.resource.unwrap_or(ResourceType::Energy) != resource_type {
                    continue;
                }
            }

            if !hauler.haul_type.contains(&order.haul_type) {
                continue;
            }

            let score = score_couple(order, &game_creep);

            if score < top_score {
                top_scorer = Some(order);
                top_score = score;
            }
        }

        if let Some(top_scorer) = top_scorer {
            let responsible_creep = hauling_cache.orders_matched_to_creeps.get(&top_scorer.id);

            if let Some(responsible_creep) = responsible_creep {
                if *responsible_creep == hauler.creep_name {
                    continue;
                }

                let responsible_creep_score = hauling_cache
                    .creep_score_matches
                    .get(responsible_creep)
                    .unwrap();

                if top_score > *responsible_creep_score {
                    hauling_cache
                        .creep_score_matches
                        .insert(hauler.creep_name.to_string(), top_score);
                    hauling_cache
                        .creep_score_matches
                        .insert(responsible_creep.to_string(), 0.0);

                    hauling_cache
                        .orders_matched_to_creeps
                        .insert(top_scorer.id, hauler.creep_name.to_string());
                    hauling_cache
                        .creeps_matched_to_orders
                        .insert(hauler.creep_name.to_string(), top_scorer.clone());

                    matched_creeps.push(hauler.creep_name.to_string());
                }
            } else {
                hauling_cache
                    .creep_score_matches
                    .insert(hauler.creep_name.to_string(), top_score);
                hauling_cache
                    .orders_matched_to_creeps
                    .insert(top_scorer.id, hauler.creep_name.to_string());
                hauling_cache
                    .creeps_matched_to_orders
                    .insert(hauler.creep_name.to_string(), top_scorer.clone());

                matched_creeps.push(hauler.creep_name.to_string());
            }
        }
    }

    let count = hauling_cache.new_orders.len();

    for (creep, order) in hauling_cache.creeps_matched_to_orders.clone().iter() {
        let creep_memory = memory.creeps.get_mut(creep).unwrap();

        let haul_task = CreepHaulTask {
            target_id: order.target,
            priority: order.priority,
            resource: order.resource.unwrap_or(ResourceType::Energy),
            amount: order.amount,
            haul_type: order.haul_type,
        };

        if let Some(reserved_order) = cache
            .rooms
            .get_mut(room_name)
            .unwrap()
            .heap_cache
            .hauling
            .reserved_orders
            .get_mut(&order.target)
        {
            reserved_order.amount_reserved += order.amount.unwrap();
            reserved_order.creeps_assigned.push(creep.to_string());
        } else {
            cache
                .rooms
                .get_mut(room_name)
                .unwrap()
                .heap_cache
                .hauling
                .reserved_orders
                .insert(
                    order.target,
                    HeapHaulingReservation {
                        target_id: order.target,
                        amount_reserved: order.amount.unwrap(),
                        creeps_assigned: vec![creep.to_string()],
                    },
                );
        }

        creep_memory.hauling_task = Some(haul_task.clone());

        let room_name = game::creeps()
            .get(creep.to_string())
            .unwrap()
            .room()
            .unwrap()
            .name();
        let room_cache = cache.rooms.get_mut(&room_name).unwrap();

        execute_order(
            &game::creeps().get(creep.to_string()).unwrap(),
            creep_memory,
            room_cache,
            &haul_task.clone(),
        );
    }

    info!(
        "  [HAULING] Matched {} haulers to {} orders in {} CPU",
        matched_creeps.len(),
        count,
        game::cpu::get_used() - starting_cpu
    );
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn score_couple(order: &RoomHaulingOrder, creep: &Creep) -> f32 {
    let creep_pos = creep.pos();
    let target = game::get_object_by_id_erased(&order.target).unwrap();

    let distance = creep_pos.get_range_to(target.pos());

    let score = order.priority + distance as f32;

    score as f32
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CreepHaulTask {
    pub fn get_target_position(&self) -> Option<Position> {
        let target = game::get_object_by_id_erased(&self.target_id);

        target.as_ref()?;

        Some(target.unwrap().pos())
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomHaulingOrder {
    pub fn get_target_position(&self) -> Option<Position> {
        let target = game::get_object_by_id_erased(&self.target);

        target.as_ref()?;

        Some(target.unwrap().pos())
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_spawn(room_cache: &mut CachedRoom) {
    let has_ff = !room_cache
        .creeps
        .creeps_of_role
        .get(&Role::FastFiller)
        .unwrap_or(&Vec::new())
        .is_empty();

    for spawn in room_cache.structures.spawns.values() {
        if spawn.store().get_free_capacity(Some(ResourceType::Energy)) == 0
            || (has_ff && room_cache.structures.containers.fast_filler.is_some())
        {
            continue;
        }

        let priority = scale_haul_priority(
            spawn.store().get_capacity(Some(ResourceType::Energy)),
            spawn.store().get_free_capacity(Some(ResourceType::Energy)) as u32,
            HaulingPriority::Spawning,
            true,
        );

        room_cache.hauling.create_order(
            spawn.raw_id(),
            Some(spawn.structure_type()),
            Some(ResourceType::Energy),
            Some(spawn.store().get_free_capacity(Some(ResourceType::Energy)) as u32),
            priority,
            HaulingType::Transfer,
        );
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_ruins(room_cache: &mut CachedRoom) {
    let ruins = &room_cache.structures.ruins;

    for ruin in ruins.values() {
        let energy_amount = ruin.store().get_used_capacity(Some(ResourceType::Energy));

        if energy_amount > 0 {
            room_cache.hauling.create_order(
                ruin.raw_id(),
                None,
                Some(ResourceType::Energy),
                Some(ruin.store().get_used_capacity(Some(ResourceType::Energy))),
                scale_haul_priority(
                    ruin.store().get_capacity(None),
                    energy_amount,
                    HaulingPriority::Ruins,
                    false,
                ),
                HaulingType::Offer,
            );
            return;
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_tombstones(room_cache: &mut CachedRoom) {
    let tombstones = &room_cache.structures.tombstones;

    for tombstone in tombstones.values() {
        let energy_amount = tombstone.store().get_used_capacity(Some(ResourceType::Energy));

        if energy_amount > 0 {
            room_cache.hauling.create_order(
                tombstone.raw_id(),
                None,
                Some(ResourceType::Energy),
                Some(tombstone.store().get_used_capacity(Some(ResourceType::Energy))),
                scale_haul_priority(
                    tombstone.store().get_capacity(None),
                    energy_amount,
                    HaulingPriority::Ruins,
                    false,
                ),
                HaulingType::Offer,
            );
            return;
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_storage(room_cache: &mut CachedRoom) {
    let storage = &room_cache.structures.storage;
    let base_hauler_count = room_cache.creeps.creeps_of_role.get(&Role::BaseHauler).unwrap_or(&Vec::new()).len();

    let priority = if base_hauler_count >= 1 {
        100.0
    } else {
        10000.0
    };

    if let Some(storage) = storage {
        if storage
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            > 0
        {
            let priority = scale_haul_priority(
                storage.store().get_capacity(Some(ResourceType::Energy)),
                storage
                    .store()
                    .get_used_capacity(Some(ResourceType::Energy)),
                HaulingPriority::Storage,
                false,
            );

            room_cache.hauling.create_order(
                storage.raw_id(),
                Some(storage.structure_type()),
                Some(ResourceType::Energy),
                Some(
                    storage
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy)),
                ),
                priority + 7.0,
                HaulingType::Offer,
            )
        }

        let fill_percent = (storage.store().get_used_capacity(None) as f32
            / storage.store().get_capacity(None) as f32) * 100.0;

        if storage.store().get_free_capacity(None) > 0 {
            room_cache.hauling.create_order(
                storage.raw_id(),
                Some(storage.structure_type()),
                None,
                Some(storage.store().get_free_capacity(None).try_into().unwrap()),
                priority - fill_percent,
                HaulingType::Transfer,
            )
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_extensions(room_cache: &mut CachedRoom) {
    let base_hauler_count = room_cache.creeps.creeps_of_role.get(&Role::BaseHauler).unwrap_or(&Vec::new()).len();

    if base_hauler_count >= 1 {
        return;
    }

    for source in room_cache.structures.extensions.values() {
        if source.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
            let priority = scale_haul_priority(
                source.store().get_capacity(Some(ResourceType::Energy)),
                source.store().get_used_capacity(Some(ResourceType::Energy)),
                HaulingPriority::Spawning,
                true,
            );

            room_cache.hauling.create_order(
                source.raw_id(),
                Some(source.structure_type()),
                Some(ResourceType::Energy),
                Some(
                    source
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        .try_into()
                        .unwrap(),
                ),
                priority,
                HaulingType::Transfer,
            );
        }
    }
}
