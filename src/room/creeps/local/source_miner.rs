use std::str::FromStr;

use log::info;
use screeps::{game, Creep, ErrorCode, HasPosition, Part, ResourceType, RoomName, SharedCreepProperties, Source};

use crate::{memory::{CreepMemory, RoomMemory, ScreepsMemory}, room::structure_cache::RoomStructureCache, traits::{creep::CreepExtensions, room::RoomExtensions}};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, structures: &RoomStructureCache) {

    if creep.near_age_death() {
        let _ = creep.say("👴", true);
        handle_death(creep, memory);
        return;
    }

    let cloned_memory = memory.clone();
    let creep_memory = memory.get_creep_mut(&creep.name());
    let room_memory = cloned_memory.get_room(&RoomName::from_str(&creep_memory.o_r).unwrap());

    if creep_memory.t_id.is_none() {
        let _ = creep.say("kurt kob", true);
        let _ = creep.suicide();
    }

    let pointer_index = creep_memory.t_id.unwrap() as usize;
    let scouted_source = &room_memory.sources[pointer_index];
    let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

    if creep_memory.n_e.unwrap_or(false) {

        harvest_source(creep, source, creep_memory);

        if creep.store().get_used_capacity(Some(ResourceType::Energy)) >= creep.store().get_capacity(Some(ResourceType::Energy)) {
            creep_memory.n_e = None;
        }
    } else {
        if !link_deposit(creep, creep_memory, &room_memory, structures) {
            drop_deposit(creep, creep_memory, structures);
        }

        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            creep_memory.n_e = Some(true);
        }
    }
}

fn harvest_source(creep: &Creep, source: Source, memory: &mut CreepMemory) {
    if !creep.pos().is_near_to(source.pos()) {
        creep.better_move_to(memory, source.pos(), 1);
    } else {
        let _ = creep.harvest(&source);
    }
}

fn link_deposit(creep: &Creep, creep_memory: &CreepMemory, room_memory: &RoomMemory, structures: &RoomStructureCache) -> bool {
    let link_pos = creep_memory.l_id;

    if let Some(links) = &room_memory.links {
        let link_id = links.get(link_pos.unwrap() as usize).unwrap();
        let link = structures.links.get(link_id).unwrap();

        if creep.pos().is_near_to(link.pos()) {
            let _ = creep.transfer(link, ResourceType::Energy, None);
            return true;
        }
        return false;
    }
    return false;
}

fn drop_deposit(creep: &Creep, creep_memory: &mut CreepMemory, structures: &RoomStructureCache) {
    //let spawn = structures.spawns.iter().next().unwrap().1;
    let spawn = structures.controller.clone().unwrap();

    //info!("Euclidean: {} - Chebeshev: {}", creep.pos().distance_to(spawn.pos()), creep.pos().get_range_to(spawn.pos()));
    if creep.pos().get_range_to(spawn.pos()) <= 2 {
        //let _ = creep.transfer(spawn, ResourceType::Energy, None);
        let _ = creep.upgrade_controller(&spawn);
    } else {
        creep.better_move_to(creep_memory, spawn.pos(), 2);
    }
}

fn handle_death(creep: &Creep, memory: &mut ScreepsMemory) {
    let creep_memory = memory.clone().get_creep(&creep.name());

    memory.get_room(&RoomName::from_str(&creep_memory.o_r).unwrap()).creeps.retain(|x| x != &creep.name());
    memory.creeps.remove(&creep.name());

    let room_memory = memory.get_room_mut(&RoomName::from_str(&creep_memory.o_r).unwrap());
    room_memory.sources[creep_memory.t_id.unwrap() as usize].assigned_creeps -= 1;
    room_memory.sources[creep_memory.t_id.unwrap() as usize].work_parts -= creep.parts_of_type(Part::Work) as u8;
}