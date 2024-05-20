use std::cmp;

use log::info;
use screeps::{Part, Room, StructureSpawn};

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, traits::room::RoomExtensions, utils::role_to_name};

pub fn formulate_miner(room: &Room, memory: &mut ScreepsMemory, spawn: StructureSpawn) -> bool {
    let mut cost = 0;
    let mut parts = Vec::new();

    let room_memory = memory.get_room_mut(&room.name());

    let needed = room.get_target_for_miner(&room_memory.clone());

    if needed.is_none() {
        let body = [Part::Move, Part::Work, Part::Carry];
        let role_name = role_to_name(Role::Upgrader);
        let name = format!("{}-{}-{}", role_name, memory.creeps.len() + 1, room.name());

        let spawn_result = spawn.spawn_creep(&body, &name);
        if spawn_result.is_ok() {
            let cmemory = CreepMemory {
                r: Role::Upgrader,
                n_e: Some(true),
                t_id: None,
                l_id: None,
                o_r: room.name().to_string(),
                p: None,
            };

            memory.create_creep(&room.name_str(), &name, &cmemory);
        }
        return false;
    }

    parts.push(Part::Move);
    parts.push(Part::Move);
    cost += 100;
    parts.push(Part::Carry);
    cost += 50;

    let energy_capacity = room.energy_capacity_available() - cost;
    let max_work_parts_makeable = energy_capacity / 100;
    let max_work_parts_needed = room_memory.sources[needed.unwrap() as usize].parts_needed();

    for _ in 0.. cmp::min(max_work_parts_makeable, max_work_parts_needed.into()) {
        parts.push(Part::Work);
        cost += 100;
    }

    let name_prefix = role_to_name(Role::Miner);
    let name = format!("{}-{}-{}", name_prefix, room_memory.creeps.len() + 1, room.name());

    if cost < room.energy_available() {
        let spawn_result = spawn.spawn_creep(&parts, &name);

        if spawn_result.is_ok() {

            info!("  [SPANWER] Spawned a new miner!");

            let cmemory = CreepMemory {
                r: crate::memory::Role::Miner,
                n_e: Some(true),
                t_id: Some(needed.unwrap()),
                l_id: None,
                o_r: room.name().to_string(),
                p: None,
            };

            memory.create_creep(&room.name_str(), &name, &cmemory);
            let room_memory = memory.get_room_mut(&room.name());

            room_memory.creeps_manufactured += 1;
            room_memory.sources.get_mut(needed.unwrap() as usize).unwrap().work_parts += parts.len() as u8 - 3;
            room_memory.sources.get_mut(needed.unwrap() as usize).unwrap().assigned_creeps += 1;
        }
    }

    false
}