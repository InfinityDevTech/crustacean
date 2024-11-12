use std::{cmp, collections::HashMap};

use log::info;
use screeps::{
    game, CircleStyle, Creep, HasId, HasPosition, Part, Position, RawObjectId, ResourceType, RoomName, SharedCreepProperties, StructureProperties, StructureType, TextStyle
};
use serde::{Deserialize, Serialize};

use crate::{
    heap, heap_cache::hauling::HeapHaulingReservation, memory::{CreepHaulTask, Role, ScreepsMemory}, room::creeps::local::hauler::execute_order, utils::{self, name_to_role, scale_haul_priority}
};

use super::{CachedRoom, RoomCache};

// Order creation ------

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

    NoDistanceCalcOffer = 4,
    NoDistanceCalcWithdraw = 5,
    NoDistanceCalcPickup = 6,
    NoDistanceCalcTransfer = 7,
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

    pub no_distance_calc: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HaulTaskRequest {
    pub creep_name: String,

    pub haul_type: Vec<HaulingType>,
    pub resource_type: Option<ResourceType>,
    pub maintain_room: Option<RoomName>,
}

impl HaulTaskRequest {
    pub fn creep_name(&mut self, creep_name: String) -> &mut Self {
        self.creep_name = creep_name;
        self
    }

    pub fn maintain_room(&mut self, maintain_room: RoomName) -> &mut Self {
        self.maintain_room = Some(maintain_room);
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
            maintain_room: None,
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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

        let mut no_distance_calc = false;
        let no_dist = [
            HaulingType::NoDistanceCalcOffer,
            HaulingType::NoDistanceCalcPickup,
            HaulingType::NoDistanceCalcTransfer,
            HaulingType::NoDistanceCalcWithdraw,
        ];

        let order_type = if no_dist.contains(&haul_type) {
            no_distance_calc = true;

            match haul_type {
                HaulingType::NoDistanceCalcOffer => HaulingType::Offer,
                HaulingType::NoDistanceCalcWithdraw => HaulingType::Withdraw,
                HaulingType::NoDistanceCalcPickup => HaulingType::Pickup,
                HaulingType::NoDistanceCalcTransfer => HaulingType::Transfer,

                _ => haul_type,
            }
        } else {
            haul_type
        };

        let mut order = RoomHaulingOrder {
            id,
            target,
            target_type,
            resource,
            amount,
            //priority: decimal.round_dp(2).try_into().unwrap(),
            priority,
            haul_type: order_type,

            no_distance_calc,
        };

        {
            // We want to drop this lock ASAP.
            let lock = heap().hauling.lock().unwrap();
            if let Some(existing_order) = lock.reserved_orders.get(&order.target) {
                let reserved_amt = existing_order.reserved_amount;

                if game::time() % 10 == 0 {
                    for creep in existing_order.creeps_assigned.iter() {
                        if game::creeps().get(creep.to_string()).is_none() {
                            continue;
                        }
                    }
                }

                if order.amount > Some(reserved_amt as u32) {
                    order.amount = Some(order.amount.unwrap() - reserved_amt as u32);

                    if order.haul_type == HaulingType::Pickup && order.target_type.is_none() {
                        order.priority = -(order.amount.unwrap_or(0) as f32);
                    }
                } else {
                    return;
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn match_haulers(room_cache: &mut RoomCache, memory: &mut ScreepsMemory, room_name: &RoomName) {
    let starting_cpu = game::cpu::get_used();
    let mut matched_creeps = Vec::new();

    if !room_cache.rooms.contains_key(room_name) {
        return;
    }

    let cache = &mut room_cache.rooms.get_mut(room_name).unwrap();
    let count = cache.hauling.new_orders.len();
    let mut saved = Vec::new();

    if cache.hauling.wanting_orders.is_empty() {
        return;
    }

    // This is wrapped because we want to drop the lock when were done with it.
    // This also highlights where we actually select the haulers.
    {
        let mut heap_hauling = heap().hauling.lock().unwrap();
        let base_hauler_count = cache
            .creeps
            .creeps_of_role(Role::BaseHauler);

        let mut orders = cache.hauling.new_orders.clone();

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

            for order in orders.values() {
                // Dont let haulers pull from storage if there is a base hauler
                // Just to avoid them getting stuck.
                if order.target_type == Some(StructureType::Storage)
                    && hauler.haul_type.contains(&HaulingType::Offer)
                    && role == Some(Role::Hauler)
                    && base_hauler_count >= 1
                {
                    continue;
                }

                if hauler.maintain_room.is_some() {
                    if let Some(target_pos) = order.get_target_position() {
                        if target_pos.room_name() != hauler.maintain_room.unwrap() {
                            continue;
                        }
                    }
                }

                // Check if the haul type actually matches what we want
                // Duh...
                if !hauler.haul_type.contains(&order.haul_type) {
                    continue;
                }

                // If the order is for a specific resource, only match it if the hauler can carry it
                // If the order doesnt contain a resource, assume energy.
                if let Some(resource_type) = hauler.resource_type {
                    if order.resource.is_some() && order.resource.unwrap() != resource_type {
                        continue;
                    }
                }

                // If the order is reserved, and the amount is reserved is
                // greater than the amount of the order, skip it, as we dont want over-hauling.
                if let Some(reserved) = heap_hauling.reserved_orders.get(&order.target) {
                    if reserved.creeps_assigned.contains(&hauler.creep_name)
                        || reserved.reserved_amount >= order.amount.unwrap() as i32
                    {
                        continue;
                    }
                }

                // Changed how some things work. The order can "Not calculate distance"
                // So that way, we can have orders that are just "highest priority" and not
                // based on distance, for say, remotes.

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
                    top_scoring_order = Some(order.clone());
                    top_score = score;
                }
            }

            // If we have a match, we do stuffs!
            if let Some(top_scorer) = top_scoring_order {
                let _responsible_creep = cache.hauling.orders_matched_to_creeps.get(&top_scorer.id);

                // If we are a better match than the other creep, we take it.
                // Fuck you other creep! This shit is mine!
                // Yes, we lose one tick on the other creep, but if its more worth it, then its fine.
                /*if let Some((responsible_creep, score)) = responsible_creep {
                    info!("Has responsible.");
                    continue;

                    if *responsible_creep == hauler.creep_name {
                        continue;
                    }

                    if top_score < *score {

                        info!("Switching tasks.");
                        let matched_order = (hauler.creep_name.to_string(), top_score);

                        cache
                            .hauling
                            .orders_matched_to_creeps
                            .insert(top_scorer.id, matched_order);

                        let order = orders.get_mut(&top_scorer.id).unwrap();

                        // If we are going to take it, decrease the resource amount by how much were going to take
                        // That way, all of our haulers dont rush for a single source at once.
                        if order.haul_type == HaulingType::Pickup {
                            let pickup_amount = cmp::min(
                                order.amount.unwrap_or(0) as i32,
                                game_creep.store().get_free_capacity(Some(
                                    order.resource.unwrap_or(ResourceType::Energy),
                                )),
                            );
                            let new_amount = order.amount.unwrap_or(0) as i32 - pickup_amount;

                            if new_amount < 0 {
                                if order.target_type != Some(StructureType::Storage) {
                                    orders.remove(&top_scorer.id);
                                }
                            } else {
                                order.amount = Some(new_amount as u32);
                            }
                        }

                        matched_creeps.push(hauler.creep_name.to_string());
                    }
                } else */{
                    let matched_order = (hauler.creep_name.to_string(), top_score);

                    cache
                        .hauling
                        .orders_matched_to_creeps
                        .insert(top_scorer.id, matched_order);

                    let order = orders.get_mut(&top_scorer.id).unwrap();

                    // If we are going to take it, decrease the resource amount by how much were going to take
                    // That way, all of our haulers dont rush for a single source at once.
                    if order.haul_type == HaulingType::Pickup {
                        let pickup_amount = cmp::min(
                            order.amount.unwrap_or(0) as i32,
                            game_creep.store().get_free_capacity(Some(
                                order.resource.unwrap_or(ResourceType::Energy),
                            )),
                        );
                        let new_amount = order.amount.unwrap_or(0) as i32 - pickup_amount;

                        if new_amount <= 0 {
                            orders.remove(&top_scorer.id);
                        } else {
                            order.amount = Some(new_amount as u32);
                        }
                    }

                    matched_creeps.push(hauler.creep_name.to_string());
                }
            }
        }

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

            let resource = if let Some(order_resource) = order.resource {
                order_resource
            } else if utils::contains_other_than(&creep.store(), ResourceType::Energy) {
                let hashed_store = utils::store_to_hashmap(&creep.store());

                let mut carrying_most_of = ResourceType::Energy;
                let mut most_carried = 0;

                for (resource, amount) in hashed_store.iter() {
                    if *amount > most_carried {
                        carrying_most_of = *resource;
                        most_carried = *amount;
                    }
                }

                carrying_most_of
            } else {
                ResourceType::Energy
            };

            // Haul task, for memory.
            let haul_task = CreepHaulTask {
                target_id: order.target,
                priority: order.priority,
                resource,
                amount: order.amount,
                haul_type: order.haul_type,
            };

            // If the target is not a storage, we reserve the order
            if order.target_type != Some(StructureType::Storage) || order.target_type.is_none() {
                // If the order is already reserved, we add the creep to the list of creeps assigned to it
                // Then increment the reserved amount.
                if let Some(reserved_order) = heap_hauling.reserved_orders.get_mut(&order.target) {
                    reserved_order.reserved_amount += carry_capacity as i32;
                    reserved_order.creeps_assigned.push(creep.name());
                } else {
                    // If the order is not reserved, we reserve it.
                    heap_hauling.reserved_orders.insert(
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
    }

    // We have the output above, because we dont want to include intents from the creeps being matched.

    let pre_exec_cpu = game::cpu::get_used();
    let size = saved.len();

    for (creep_name, haul_task) in saved {
        let creep = game::creeps().get(creep_name.clone()).unwrap();

        execute_order(&creep, memory, room_cache, &haul_task);
    }

    info!(
        "[HAULING] Room {} matched {} haulers to {} orders in {:.2} CPU. Ran {} haulers in {:.2} CPU.",
        room_name,
        matched_creeps.len(),
        count,
        game::cpu::get_used() - starting_cpu,
        size,
        game::cpu::get_used() - pre_exec_cpu,
    );
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn score_couple(order: &RoomHaulingOrder, creep: &Creep) -> f32 {
    if order.no_distance_calc {
        order.priority
    } else {
        let creep_pos = creep.pos();
        let order_pos = order.get_target_position().unwrap();

        let distance = creep_pos.get_range_to(order_pos);

        order.priority + distance as f32
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn clean_heap_hauling(memory: &mut ScreepsMemory) {
    let mut to_delete = Vec::new();

    let mut lock = heap().hauling.lock().unwrap();

    for hauling_order in lock.reserved_orders.values_mut() {
        let game_obj = game::get_object_by_id_erased(&hauling_order.target_id);
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
                carry_total += game_creep
                    .body()
                    .iter()
                    .filter(|part| part.part() == Part::Carry)
                    .count()
                    * 50;
            } else {
                carry_total += 200;
            }
        }

        hauling_order.reserved_amount -= carry_total as i32;
        if hauling_order.reserved_amount <= 0
            || hauling_order.reserved_amount > hauling_order.order_amount
            || game_obj.is_none()
            || hauling_order.creeps_assigned.is_empty()
        {
            to_delete.push(hauling_order.target_id);
        }
    }

    // Delete these marked orders
    for target in to_delete {
        lock.reserved_orders.remove(&target);
    }
}


impl CreepHaulTask {
    pub fn get_target_position(&self) -> Option<Position> {
        let target = game::get_object_by_id_erased(&self.target_id);

        target.as_ref()?;

        Some(target.unwrap().pos())
    }
}

impl RoomHaulingOrder {
    pub fn get_target_position(&self) -> Option<Position> {
        let target = game::get_object_by_id_erased(&self.target);

        target.as_ref()?;

        Some(target.unwrap().pos())
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_spawn(room_cache: &mut CachedRoom) {
    let has_ff = room_cache
        .creeps
        .creeps_of_role(Role::FastFiller) > 0;

    let has_base_hauler = room_cache
        .creeps
        .creeps_of_role(Role::BaseHauler) > 0;

    for spawn in room_cache.structures.spawns.values() {
        if spawn.store().get_free_capacity(Some(ResourceType::Energy)) == 0 || (has_ff && (room_cache.structures.containers().fast_filler.is_some() || (room_cache.structures.links().fast_filler.is_some() && room_cache.rcl >= 7) )) || has_base_hauler {
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
    let tombstones = &room_cache.structures.tombstones();

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
            // Priority is 3k because, if there is dropped energy greater than this
            // We should probably pick it up...
            room_cache.hauling.create_order(
                storage.raw_id(),
                Some(storage.structure_type()),
                Some(ResourceType::Energy),
                Some(
                    storage
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy)),
                ),
                -1000.0,
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
                HaulingType::NoDistanceCalcTransfer,
            )
        }

        room_cache.stats.energy.stored = storage.store().get_used_capacity(Some(ResourceType::Energy));
    }
}

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
            ) - 5.0;

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
