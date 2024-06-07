use std::cmp;

use log::info;
use screeps::{game, HasId, Part, ResourceType, Room};

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::tick_cache::{
        hauling::{HaulingPriority, HaulingType},
        RoomCache,
    },
    traits::room::RoomExtensions,
    utils::{role_to_name, scale_haul_priority},
};

pub fn formulate_miner(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let mut cost = 0;
    let mut parts = Vec::new();

    let needed = room.get_target_for_miner(room, cache);

    let spawn = cache.structures.spawns.iter().next().unwrap().1;

    let fastfiller_count = cache
    .creeps
    .creeps_of_role
    .get(&Role::FastFiller)
    .unwrap_or(&vec![])
    .len();

    let miner_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Miner)
        .unwrap_or(&vec![])
        .len();

        let hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&vec![])
        .len();

        let giftdrop_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::GiftBasket)
        .unwrap_or(&vec![])
        .len();

    if fastfiller_count == 0 && spawn.store().get_used_capacity(Some(ResourceType::Energy)) < 300 {
        let priority = scale_haul_priority(
            spawn.store().get_capacity(None),
            spawn.store().get_free_capacity(None) as u32,
            HaulingPriority::Spawning,
            true
        );

        cache.hauling.create_order(
            spawn.raw_id(),
            Some(ResourceType::Energy),
                Some(spawn.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap()),
            priority,
            HaulingType::Transfer,
        );
    }

    if needed.is_none() || (miner_count >= 1 && hauler_count == 0) {
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
        let bulldozer_count = cache
            .creeps
            .creeps_of_role
            .get(&Role::Bulldozer)
            .unwrap_or(&vec![])
            .len();
        let scout_count = cache
            .creeps
            .creeps_of_role
            .get(&Role::Scout)
            .unwrap_or(&vec![])
            .len();

        if hauler_count < 10 && (fastfiller_count >= 1 || hauler_count == 0) {
            let mut body = Vec::new();
            let cost = 100;

            let max = if hauler_count < 3 {
                room.energy_available()
            } else {
                room.energy_capacity_available()
            };

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
                    scout_target: None,
                    fastfiller_container: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);

                return true;
            }
        } else if fastfiller_count < 2 {
            let body = vec![Part::Move, Part::Carry];

            let role_name = role_to_name(Role::FastFiller);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    link_id: None,
                    hauling_task: None,
                    fastfiller_container: None,
                    scout_target: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);

                return true;
            }
        } else if upgrader_count < 4 {
            let mut body = Vec::new();
            let cost = 300;
            let max = room.energy_capacity_available();
            let max_multipliable = max / cost;
            let mut current = 0;

            loop {
                if current >= max_multipliable { break }
                body.push(Part::Work);
                body.push(Part::Work);
                body.push(Part::Carry);
                body.push(Part::Move);
                current += 1;
            }
            let role_name = role_to_name(Role::Upgrader);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    fastfiller_container: None,
                    link_id: None,
                    scout_target: None,
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
                if current >= max_multipliable { break }
                body.push(Part::Work);
                body.push(Part::Carry);
                body.push(Part::Move);
                body.push(Part::Move);
                current += 1;
            }
            let role_name = role_to_name(Role::Builder);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    fastfiller_container: None,
                    link_id: None,
                    hauling_task: None,
                    scout_target: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);

                return true;
            }
        } else if scout_count < 2 {
            let body = vec![Part::Move];
            let role_name = role_to_name(Role::Scout);

            let name = format!("{}-{}-{}", role_name, game::time(), room.name());
            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    fastfiller_container: None,
                    link_id: None,
                    hauling_task: None,
                    scout_target: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);

                return true;
            }

        } else if bulldozer_count < 5 && game::flags().get("bulldozeRoom".to_string()).is_some() {

            info!("Bulldozing room");

            let mut body = Vec::new();
            let cost = 130;
            let max = room.energy_capacity_available();
            let max_multipliable = max / cost;
            let mut current = 0;

            loop {
                if current >= max_multipliable { break }
                body.push(Part::Attack);
                body.push(Part::Move);
                current += 1;
            }

            let role_name = role_to_name(Role::Bulldozer);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());

            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    fastfiller_container: None,
                    link_id: None,
                    scout_target: None,
                    hauling_task: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);

                return true;
            }
        } else if giftdrop_count < 3 && game::flags().get("bulldozeRoom".to_string()).is_some() {
            let body = vec![Part::Move, Part::Carry];

            let role_name = role_to_name(Role::GiftBasket);
            let name = format!("{}-{}-{}", role_name, game::time(), room.name());
            let spawn_result = spawn.spawn_creep(&body, &name);
            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    needs_energy: None,
                    task_id: None,
                    fastfiller_container: None,
                    link_id: None,
                    scout_target: None,
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
        let max_work_parts_makeable = (energy_capacity as f32 / 100.0).floor() as u32;
        let max_work_parts_needed = cache.resources.sources[needed.unwrap() as usize].parts_needed();

        for _ in 0..cmp::min(max_work_parts_makeable, (max_work_parts_needed + 2).into()) {
            if parts.len() % 4 == 0 {
                parts.push(Part::Move);
                cost += 50;
            } else {
                parts.push(Part::Work);
                cost += 100;
            }
        }

        let name_prefix = role_to_name(Role::Miner);
        let name = format!("{}-{}-{}", name_prefix, game::time(), room.name());

        if cost <= room.energy_available() {
            let spawn_result = spawn.spawn_creep(&parts, &name);

            if spawn_result.is_ok() {
                let cmemory = CreepMemory {
                    needs_energy: Some(true),
                    task_id: Some(needed.unwrap().into()),
                    fastfiller_container: None,
                    hauling_task: None,
                    link_id: None,
                    scout_target: None,
                    owning_room: room.name().to_string(),
                    path: None,
                };

                memory.create_creep(&room.name_str(), &name, cmemory);
            }
        }
    }
    false

}
