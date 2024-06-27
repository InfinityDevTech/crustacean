use screeps::{game, Creep, HasId, HasPosition, ResourceType, SharedCreepProperties, StructureType};

use crate::{memory::{CreepMemory, ScreepsMemory}, room::cache::tick_cache::{hauling::{HaulTaskRequest, HaulingType}, CachedRoom, RoomCache}, traits::creep::CreepExtensions, utils::get_rampart_repair_rcl};
use wasm_bindgen::JsCast;
use super::hauler;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_repairer(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let room_cache = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if creep_memory.needs_energy.unwrap_or(false) {
        get_energy(creep, creep_memory, cache);

        if creep.store().get_free_capacity(None) == 0 {
            creep_memory.needs_energy = None;
        }

        return;
    }

    if let Some(repair_task) = creep_memory.repair_target {
        let repairable = game::get_object_by_id_typed(&repair_task);

        // So we evenly distribute the repair tasks.
        if repairable.is_none() || game::time() % 10 == 0 {
            creep_memory.repair_target = None;
            creep_memory.path = None;
            return;
        }
        let repairable = repairable.unwrap();

        let max = if repairable.structure_type() == StructureType::Rampart {
            let controller = room_cache.structures.controller.as_ref().unwrap();
            get_rampart_repair_rcl(controller.controller.level())
        } else {
            repairable.hits_max()
        };

        // Repair ramparts to 110% of our RCL max, just to avoid repairing them every 3 seconds
        // Other structures, 100%
        if ( repairable.structure_type() == StructureType::Rampart && repairable.hits() as f32 >= (max as f32) * 1.1) || ( repairable.structure_type() != StructureType::Rampart && repairable.hits() as f32 >= max as f32) {
            creep_memory.repair_target = None;
            creep_memory.path = None;
            return;
        }

        let repairable = repairable.unchecked_ref::<screeps::StructureRoad>();
        if creep.pos().get_range_to(repairable.pos()) < 3 {
            let _ = creep.say("🔧", false);
            let _ = creep.repair(repairable);
        } else {
            let _ = creep.say("🚚", false);
            creep.better_move_to(creep_memory, cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(), repairable.pos(), 2, Default::default());

            return;
        }
    } else if !get_repair_task(creep, creep_memory, room_cache) {
        let _ = creep.say("❓", false);
        return;
    } else {
        run_repairer(creep, memory, cache);
        return;
    }

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creep_memory.needs_energy = Some(true);
    } else {
        creep_memory.needs_energy = None;
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_repair_task(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut CachedRoom) -> bool {
    let mut lowest_rank = f32::MAX;
    let mut lowest_rank_id = None;

    if cache.structures.needs_repair.is_empty() {
        return false;
    }

    for repairable_structure in &cache.structures.needs_repair {
        let structure = repairable_structure.as_structure();
        if let Some(repairable) = repairable_structure.as_repairable() {
            let max_hits = if structure.structure_type() == StructureType::Rampart {
                100_000
            } else {
                repairable.hits_max()
            };
            let health_percentage = (repairable.hits() as f32 / max_hits as f32) * 100.0;
            let distance = creep.pos().get_range_to(repairable_structure.pos());

            if health_percentage >= 100.0 {
                continue;
            }

            let rank = health_percentage + distance as f32;

            if rank < lowest_rank {
                lowest_rank = rank;
                lowest_rank_id = Some(structure.id());
            }
        }
    }

    if let Some(lowest_rank_id) = lowest_rank_id {
        creep_memory.repair_target = Some(lowest_rank_id);
        true
    } else {
        false
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_energy(creep: &Creep, memory: &mut CreepMemory, cache: &mut RoomCache) {
    if let Some(hauling_task) = memory.hauling_task.clone() {
        hauler::execute_order(creep, memory, cache, &hauling_task);
    } else {
        let cache = cache.rooms.get_mut(&memory.owning_room).unwrap();
        cache.hauling.wanting_orders.push(HaulTaskRequest::default().creep_name(creep.name()).resource_type(ResourceType::Energy).haul_type(vec![HaulingType::Pickup, HaulingType::Withdraw, HaulingType::Offer]).finish());
    }
}