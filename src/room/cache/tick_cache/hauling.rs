use std::collections::HashMap;

use log::info;
use screeps::{
    game, CircleStyle, Creep, HasId, HasPosition, Part, Position, RawObjectId, ResourceType, RoomName, SharedCreepProperties, StructureProperties, StructureType, TextStyle
};
use serde::{Deserialize, Serialize};

use crate::{
    heap, memory::{CreepHaulTask, Role, ScreepsMemory},
    room::
        cache::heap_cache::
            hauling::HeapHaulingReservation
        
    ,
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

    orders_matched_to_creeps: HashMap<u32, (String, f32)>,

    iterator_salt: u32,
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl HaulingCache {
    pub fn new() -> HaulingCache {
        HaulingCache {
            new_orders: HashMap::new(),
            current_id_index: game::time(),
            haulers: Vec::new(),

            wanting_orders: Vec::new(),

            orders_matched_to_creeps: HashMap::new(),

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

        let mut order = RoomHaulingOrder {
            id,
            target,
            target_type,
            resource,
            amount,
            //priority: decimal.round_dp(2).try_into().unwrap(),
            priority,
            haul_type,
        };

        if let Some(target_pos) = &order.get_target_position() {
            let mut lock = heap().rooms.lock().unwrap();
            if let Some(heap_cache) = lock.get_mut(&target_pos.room_name()) {
                if let Some(existing_order) = heap_cache.hauling.reserved_orders.get(&order.target)
                {
                    let reserved_amt = existing_order.order_amount;

                    if game::time() % 10 == 0 {
                        for creep in existing_order.creeps_assigned.iter() {
                            if game::creeps().get(creep.to_string()).is_none() {
                                continue;
                            }
                        }
                    }

                    if order.amount > Some(reserved_amt as u32) {
                        order.amount = Some(order.amount.unwrap() - reserved_amt as u32);
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
                position.y().u8() as f32 - 0.25,
                format!("T {:.2}", order.priority),
                Some(TextStyle::default().color("#ff0000")),
            );
        } else {
            room_visual.text(
                position.x().u8() as f32,
                position.y().u8() as f32 + 0.5,
                format!("P {:.2}", order.priority),
                Some(TextStyle::default().color("#00ff00")),
            );
        }

        self.new_orders.insert(id, order);
    }
}

pub fn match_haulers(room_cache: &mut RoomCache, memory: &mut ScreepsMemory, room_name: &RoomName) {
    let starting_cpu = game::cpu::get_used();
    let mut matched_creeps = Vec::new();

    let cache = &mut room_cache.rooms.get_mut(room_name).unwrap();
    let base_hauler_count = cache.creeps.creeps_of_role.get(&Role::BaseHauler).unwrap_or(&Vec::new()).len();

    // CPU saver, dont execute if theres no haulers
    if cache.hauling.wanting_orders.is_empty() {
        info!("  [HAULING] No haulers wanting orders");
        return;
    }

    // Where the magic happens, each hauler runs through each order
    for hauler in cache.hauling.wanting_orders.iter() {
        let game_creep = game::creeps().get(hauler.creep_name.to_string()).unwrap();

        let mut top_scoring_order = None;
        let mut top_score = f32::MAX;

        let role = name_to_role(&hauler.creep_name);

        for order in cache.hauling.new_orders.values() {
            // Dont let haulers pull from storage if there is a base hauler
            // Just to avoid them getting stuck.
            if order.target_type == Some(StructureType::Storage)
                && hauler.haul_type.contains(&HaulingType::Offer)
                && role == Some(Role::Hauler)
                && base_hauler_count >= 1
            {
                continue;
            }

            // Check if the haul type actually matches what we want
            // Duh...
            if !hauler.haul_type.contains(&order.haul_type) {
                continue;
            }

            // If the order is for a specific resource, only match it if the hauler can carry it
            // If the order doesnt contain a resource, assume energy.
            if let Some(resource_type) = hauler.resource_type {
                if order.resource.unwrap_or(ResourceType::Energy) != resource_type {
                    continue;
                }
            }

            // If the order is reserved, and the amount is reserved is
            // greater than the amount of the order, skip it, as we dont want over-hauling.
            if let Some(reserved) = cache.heap_cache.hauling.reserved_orders.get(&order.target) {
                if reserved.creeps_assigned.contains(&hauler.creep_name)
                    || reserved.reserved_amount >= order.amount.unwrap() as i32
                {
                    continue;
                }
            }

            // If the order is a pickup order, these get ranked differently
            // These get ranked based off of energy count. Its flipped for
            // compatiblity with the other ranking system, described below.
            if order.haul_type == HaulingType::Pickup && role == Some(Role::Hauler) {
                let score = order.amount.unwrap_or(0) as f32;

                let score = -score;

                if score < top_score {
                    top_scoring_order = Some(order);
                    top_score = score;
                }
                continue;
            }

            // This function scores based off of
            // priority + distance
            // and if its the lowest, we take it.
            // As its the highest priority, and the closest.
            let mut score = score_couple(order, &game_creep);

            // We dont want the hauler transferring to the storage
            // if the base hauler doesnt exist, its to influence sending
            // to extensions and upgrading. Treating it more of a "last resort"
            if order.target_type == Some(StructureType::Storage)
                && hauler.haul_type.contains(&HaulingType::Transfer)
                && role == Some(Role::Hauler)
                && base_hauler_count == 0
            {
                score += 1000.0;
            }

            if score < top_score {
                top_scoring_order = Some(order);
                top_score = score;
            }
        }

        // If we have a match, we do stuffs!
        if let Some(top_scorer) = top_scoring_order {
            let responsible_creep = cache.hauling.orders_matched_to_creeps.get(&top_scorer.id);

            // If we are a better match than the other creep, we take it.
            // Fuck you other creep! This shit is mine!
            // Yes, we lose one tick on the other creep, but if its more worth it, then its fine.
            if let Some((responsible_creep, score)) = responsible_creep {
                if *responsible_creep == hauler.creep_name {
                    continue;
                }

                if top_score < *score {
                    let matched_order = (hauler.creep_name.to_string(), top_score);

                    cache.hauling.orders_matched_to_creeps.insert(top_scorer.id, matched_order);

                    matched_creeps.push(hauler.creep_name.to_string());
                }
            } else {
                let matched_order = (hauler.creep_name.to_string(), top_score);

                cache.hauling.orders_matched_to_creeps.insert(top_scorer.id, matched_order);

                matched_creeps.push(hauler.creep_name.to_string());
            }
        }
    }

    let count = cache.hauling.new_orders.len();
    let mut saved = Vec::new();

    // For the matched creeps, we assign their tasks in memory
    // This is also where we reserve the orders, so other haulers dont take them.
    for (order_id, (creep, _score)) in cache.hauling.orders_matched_to_creeps.clone().iter() {
        let creep_memory = memory.creeps.get_mut(creep).unwrap();
        let creep = game::creeps().get(creep.to_string()).unwrap();

        let order = cache.hauling.new_orders.get(order_id).unwrap();

        // Get the amount of resources the creep can carry
        let carry_capacity = creep
            .store()
            .get_free_capacity(Some(order.resource.unwrap_or(ResourceType::Energy)));

        // Haul task, for memory.
        let haul_task = CreepHaulTask {
            target_id: order.target,
            priority: order.priority,
            resource: order.resource.unwrap_or(ResourceType::Energy),
            amount: order.amount,
            haul_type: order.haul_type,
        };

        // If the target is not a storage, we reserve the order
        if order.target_type != Some(StructureType::Storage) {

            // If the order is already reserved, we add the creep to the list of creeps assigned to it
            // Then increment the reserved amount.
            if let Some(reserved_order) = cache
                .heap_cache
                .hauling
                .reserved_orders
                .get_mut(&order.target)
            {
                reserved_order.reserved_amount += carry_capacity as i32;
                reserved_order.creeps_assigned.push(creep.name());
            } else {
                // If the order is not reserved, we reserve it.
                cache.heap_cache.hauling.reserved_orders.insert(
                    order.target,
                    HeapHaulingReservation {
                        target_id: order.target,
                        reserved_amount: carry_capacity as i32,
                        creeps_assigned: vec![creep.name()],
                        order_amount: order.amount.unwrap_or(0) as i32,
                    },
                );
            }
        }

        // Set it in memory, so the creep can execute it.
        creep_memory.hauling_task = Some(haul_task.clone());

        saved.push((creep.name(), haul_task));
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

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn clean_heap_hauling(cache: &mut RoomCache, memory: &mut ScreepsMemory) {
    for room in &cache.my_rooms {
        let room_cache = cache.rooms.get_mut(room).unwrap();
        let mut to_delete = Vec::new();

        for hauling_order in room_cache.heap_cache.hauling.reserved_orders.values_mut() {
            let mut removed = Vec::new();
            // Remove creeps that are dead or have no task assigned
            // Then add them to the ^^^^^ removed list, so we can calculate carry parts.
            hauling_order.creeps_assigned.retain(|creep| {
                if let Some(creep_memory) = memory.creeps.get_mut(creep) {
                    if let Some(task_id) = creep_memory.hauling_task.as_ref() {
                        let res = task_id.target_id == hauling_order.target_id;

                        if !res {
                            removed.push(creep.to_string());

                            false
                        } else {
                            true
                        }
                    } else {
                        removed.push(creep.to_string());

                        false
                    }
                } else {
                    removed.push(creep.to_string());

                    false
                }
            });

            // Calculate the amount of resources the creeps can carry
            let mut carry_total = 0;
            for creep in removed {
                if let Some(game_creep) = game::creeps().get(creep) {
                    carry_total += game_creep.body().iter().filter(|part| part.part() == Part::Carry).count() * 50;
                } else {
                    carry_total += 200;
                }
            }

            hauling_order.reserved_amount -= carry_total as i32;
            if hauling_order.reserved_amount <= 0 || hauling_order.reserved_amount > hauling_order.order_amount {
                to_delete.push(hauling_order.target_id);
            }
        }

        // Delete these marked orders
        for target in to_delete {
            room_cache.heap_cache.hauling.reserved_orders.remove(&target);
        }
    }
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
                -20000.0,
                HaulingType::Offer,
            );
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_tombstones(room_cache: &mut CachedRoom) {
    let tombstones = &room_cache.structures.tombstones;

    for tombstone in tombstones.values() {
        let energy_amount = tombstone
            .store()
            .get_used_capacity(Some(ResourceType::Energy));

        if energy_amount > 0 {
            room_cache.hauling.create_order(
                tombstone.raw_id(),
                None,
                Some(ResourceType::Energy),
                Some(
                    tombstone
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy)),
                ),
                scale_haul_priority(
                    tombstone.store().get_capacity(None),
                    energy_amount,
                    HaulingPriority::Ruins,
                    false,
                ),
                HaulingType::Offer,
            );
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_storage(room_cache: &mut CachedRoom) {
    let storage = &room_cache.structures.storage;
    let base_hauler_count = room_cache
        .creeps
        .creeps_of_role
        .get(&Role::BaseHauler)
        .unwrap_or(&Vec::new())
        .len();

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
            room_cache.hauling.create_order(
                storage.raw_id(),
                Some(storage.structure_type()),
                Some(ResourceType::Energy),
                Some(
                    storage
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy)),
                ),
                -5.0,
                HaulingType::Offer,
            )
        }

        let fill_percent = (1.0
            - (storage.store().get_used_capacity(None) as f32
                / storage.store().get_capacity(None) as f32))
            * 100.0;

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
    let base_hauler_count = room_cache
        .creeps
        .creeps_of_role
        .get(&Role::BaseHauler)
        .unwrap_or(&Vec::new())
        .len();

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
