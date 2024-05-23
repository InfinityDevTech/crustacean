use std::str::FromStr;

use screeps::{game, Creep, ErrorCode, HasPosition, MaybeHasId, Part, ResourceType, RoomName, SharedCreepProperties, Source};

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, room::cache::{hauling::{HaulingPriority, HaulingType}, RoomCache}, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {

    if creep.near_age_death() {
        let _ = creep.say("👴", true);
        handle_death(creep, memory);
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let room_memory = memory.rooms.get(&RoomName::from_str(&creep_memory.owning_room).unwrap()).unwrap();

    if creep_memory.task_id.is_none() {
        let _ = creep.say("kurt kob", true);
        let _ = creep.suicide();
    }

    let pointer_index = creep_memory.task_id.unwrap() as usize;
    let scouted_source = &room_memory.sources[pointer_index];
    let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

    if creep_memory.needs_energy.unwrap_or(false) {

        harvest_source(creep, source, creep_memory);

        if creep.store().get_used_capacity(Some(ResourceType::Energy)) >= creep.store().get_capacity(Some(ResourceType::Energy)) {
            creep_memory.needs_energy = None;
            if !link_deposit(creep, creep_memory, cache) {
                drop_deposit(creep, creep_memory, cache);
            }
        }
    } else {
        if !link_deposit(creep, creep_memory, cache) {
            drop_deposit(creep, creep_memory, cache);
        }

        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            creep_memory.needs_energy = Some(true);
            harvest_source(creep, source, creep_memory);
        }
    }
}

fn needs_haul_manually(creep: &Creep, creep_memory: &mut CreepMemory, cache: &RoomCache) -> bool {
    let count = if let Some(creeps) = cache.creeps.creeps_of_role.get(&Role::Hauler) {
        creeps.len()
    } else {
        0
    };

    if count == 0 {
        let _ = creep.say("🚚", true);

        let spawn = cache.structures.spawns.clone().into_iter().next().unwrap().1;
        if creep.transfer(&spawn, ResourceType::Energy, None) == Err(ErrorCode::NotInRange) {
            creep.better_move_to(creep_memory, spawn.pos(), 1);
        }
        return true;
    }
    false
}

fn harvest_source(creep: &Creep, source: Source, memory: &mut CreepMemory) {
    if !creep.pos().is_near_to(source.pos()) {
        creep.better_move_to(memory, source.pos(), 1);
    } else {
        let _ = creep.harvest(&source);
    }
}

fn link_deposit(creep: &Creep, creep_memory: &mut CreepMemory, cache: &RoomCache) -> bool {
    let link_id = creep_memory.link_id;

    if let Some(linkid) = link_id {
        let link = cache.structures.links.get(&linkid).unwrap();

        if creep.pos().is_near_to(link.pos()) {
            let _ = creep.transfer(link, ResourceType::Energy, Some(creep.store().get_used_capacity(Some(ResourceType::Energy))));
        } else {
            return false;
        }
    }
    false
}

fn drop_deposit(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut RoomCache) {

    if needs_haul_manually(creep, creep_memory, cache) {
        return;
    }

    let amount = creep.store().get_used_capacity(Some(ResourceType::Energy));

    //let mut mutable = cache.hauling.borrow_mut();
    cache.hauling.create_order(creep.try_raw_id().unwrap(), ResourceType::Energy, amount, HaulingPriority::Energy, HaulingType::Pickup);
}

fn handle_death(creep: &Creep, memory: &mut ScreepsMemory) {
    let CreepMemory {task_id, owning_room, ..} = memory.creeps.get(&creep.name()).unwrap().clone();

    let room_memory = memory.rooms.get_mut(&RoomName::from_str(&owning_room).unwrap()).unwrap();

    room_memory.creeps.retain(|x| x != &creep.name());
    memory.creeps.remove(&creep.name());

    room_memory.sources[task_id.unwrap() as usize].assigned_creeps -= 1;
    room_memory.sources[task_id.unwrap() as usize].work_parts -= creep.parts_of_type(Part::Work) as u8;
}