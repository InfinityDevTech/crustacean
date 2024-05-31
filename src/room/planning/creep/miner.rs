use std::cmp;

use screeps::{game, HasId, Part, ResourceType, Room};

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::tick_cache::{
        hauling::{HaulingPriority, HaulingType},
        RoomCache,
    },
    traits::room::RoomExtensions,
    utils::role_to_name,
};

pub fn formulate_miner(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let mut cost = 0;
    let mut parts = Vec::new();

    let needed = room.get_target_for_miner(room, cache);

    let spawn = cache.structures.spawns.iter().next().unwrap().1;

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
        let builder_count = cache
            .creeps
            .creeps_of_role
            .get(&Role::Builder)
            .unwrap_or(&vec![])
            .len();

        if hauler_count < 12 {
            let mut body = Vec::new();
            let cost = 100;
            let max = room.energy_capacity_available();
            let max_multipliable = max / cost;
            let mut current = 0;

            loop {
                if current >=max_multipliable { break }
                body.push(Part::Carry);
                body.push(Part::Move);
                current += 1;
            }

            let role_name = role_to_name(Role::Hauler);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
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
            let mut body = Vec::new();
            let cost = 300;
            let max = room.energy_capacity_available();
            let max_multipliable = max / cost;
            let mut current = 0;

            loop {
                if current >=max_multipliable { break }
                body.push(Part::Work);
                body.push(Part::Work);
                body.push(Part::Carry);
                body.push(Part::Move);
                current += cost;
            }
            let role_name = role_to_name(Role::Upgrader);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
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
        } else if builder_count < 5 {
            let mut body = Vec::new();
            let cost = 250;
            let max = room.energy_capacity_available();
            let max_multipliable = max / cost;
            let mut current = 0;

            loop {
                if current >=max_multipliable { break }
                body.push(Part::Work);
                body.push(Part::Carry);
                body.push(Part::Move);
                body.push(Part::Move);
                current += cost;
            }
            let role_name = role_to_name(Role::Builder);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
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
        parts.push(Part::Carry);
        parts.push(Part::Work);
        cost += 200;

        let energy_capacity = room.energy_available() - cost;
        let max_work_parts_makeable = (energy_capacity as f32 / 100.0).ceil() as u32;
        let max_work_parts_needed = cache.structures.sources[needed.unwrap() as usize].parts_needed();

        for _ in 0..cmp::min(max_work_parts_makeable, (max_work_parts_needed + 2).into()) {
            parts.push(Part::Work);
            cost += 100;
        }

        let name_prefix = role_to_name(Role::Miner);
        let name = format!("{}-{}-{}", name_prefix, game::time(), room.name());

        if cost <= room.energy_available() {
            let spawn_result = spawn.spawn_creep(&parts, &name);

            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    needs_energy: Some(true),
                    task_id: Some(needed.unwrap().into()),
                    hauling_task: None,
                    link_id: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);
            }
        }
    }
    false

}
