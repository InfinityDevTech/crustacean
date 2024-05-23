use std::cmp;

use screeps::{game, HasId, Part, ResourceType, Room};

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::{
        hauling::{HaulingPriority, HaulingType},
        RoomCache,
    },
    traits::room::RoomExtensions,
    utils::role_to_name,
};

pub fn formulate_miner(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let mut cost = 0;
    let mut parts = Vec::new();

    let spawn = cache.structures.spawns.iter().next().unwrap().1;

    let needed = room.get_target_for_miner(memory.rooms.get(&room.name()).unwrap());

    let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

    if spawn.store().get_used_capacity(Some(ResourceType::Energy)) < 300 {
        cache.hauling.create_order(
            spawn.raw_id(),
            ResourceType::Energy,
            300,
            HaulingPriority::Energy,
            HaulingType::Transfer,
        );
    }

    if needed.is_none() {
        let hauler_count = cache
            .creeps
            .creeps_of_role
            .get(&Role::Hauler)
            .unwrap_or(&vec![])
            .len();
        let upgrader_count = cache
            .creeps
            .creeps_of_role
            .get(&Role::Upgrader)
            .unwrap_or(&vec![])
            .len();

        if hauler_count < 6 {
            let body = [Part::Carry, Part::Move, Part::Move];
            let role_name = role_to_name(Role::Hauler);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    role: Role::Hauler,
                    needs_energy: None,
                    task_id: None,
                    link_id: None,
                    hauling_task: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);

                return true;
            }
        } else if upgrader_count < 6 {
            let body = [Part::Carry, Part::Work, Part::Move];
            let role_name = role_to_name(Role::Upgrader);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    role: Role::Upgrader,
                    needs_energy: None,
                    task_id: None,
                    link_id: None,
                    hauling_task: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);

                return true;
            }
        }
    } else {
        parts.push(Part::Move);
        cost += 50;
        parts.push(Part::Carry);
        cost += 50;

        let energy_capacity = room.energy_capacity_available() - cost;
        let max_work_parts_makeable = energy_capacity / 100;
        let max_work_parts_needed = room_memory.sources[needed.unwrap() as usize].parts_needed();

        for _ in 0..cmp::min(max_work_parts_makeable, max_work_parts_needed.into()) {
            parts.push(Part::Work);
            cost += 100;
        }

        let name_prefix = role_to_name(Role::Miner);
        let name = format!("{}-{}-{}", name_prefix, game::time(), room.name());

        if cost <= room.energy_available() {
            let spawn_result = spawn.spawn_creep(&parts, &name);

            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    role: crate::memory::Role::Miner,
                    needs_energy: Some(true),
                    task_id: Some(needed.unwrap().into()),
                    hauling_task: None,
                    link_id: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);
                let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

                room_memory.creeps_manufactured += 1;
                room_memory
                    .sources
                    .get_mut(needed.unwrap() as usize)
                    .unwrap()
                    .work_parts += parts.iter().filter(|x| **x == Part::Work).count() as u8;
                room_memory
                    .sources
                    .get_mut(needed.unwrap() as usize)
                    .unwrap()
                    .assigned_creeps += 1;
                return true;
            }
        }
    }
    false
}
