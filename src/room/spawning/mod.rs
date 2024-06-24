use std::cmp::{self, min};

use log::info;
use screeps::{find, game, HasPosition, Part, Room, SharedCreepProperties};
use spawn_manager::SpawnManager;

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory}, movement::move_target::{MoveOptions, MoveTarget}, utils::get_body_cost
};

use super::cache::{self, tick_cache::{CachedRoom, RoomCache}};

pub mod creep_sizing;
pub mod spawn_manager;

// TODO:
//  Add required role counts
//  Fuck this shit man, this looks like ass
//  Tweak a shit load of numbers. Spawning needs to be PERFECT.
//  TODO!!! Fix the double remote spawning bug!! This is BAD...

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn handle_spawning(room: &Room, cache: &mut RoomCache, memory: &mut ScreepsMemory) {
    let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

    let mut spawn_manager = SpawnManager::new(&room.name(), room_cache);

    miner(room, room_cache, &mut spawn_manager);
    base_hauler(room, room_cache, &mut spawn_manager, memory);
    fast_filler(room, room_cache, &mut spawn_manager);
    flag_attacker(room, room_cache, &mut spawn_manager);
    builder(room, room_cache, &mut spawn_manager);
    repairer(room, room_cache, &mut spawn_manager);
    upgrader(room, room_cache, &mut spawn_manager);
    scout(room, room_cache, &mut spawn_manager);

    remote_harvester(room, cache, memory, &mut spawn_manager);

    hauler(room, cache, &mut spawn_manager, memory);

    spawn_manager.run_spawning(room, memory);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn flag_attacker(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    if let Some(flag) = game::flags().get("bulldozeRoom".to_string()) {
        let attackers = cache
            .creeps
            .creeps_of_role
            .get(&Role::Bulldozer)
            .unwrap_or(&Vec::new())
            .len();

        let unclaimer = cache
            .creeps
            .creeps_of_role
            .get(&Role::Unclaimer)
            .unwrap_or(&Vec::new())
            .len();

        let mut should_spawn_unclaimer = false;
        if let Some(room) = flag.room() {
            if room.find(find::HOSTILE_CREEPS, None).is_empty() && room.find(find::HOSTILE_SPAWNS, None).is_empty() && room.find(find::HOSTILE_STRUCTURES, None).is_empty() {
                should_spawn_unclaimer = true;
            }
        }

        if attackers >= 4 && unclaimer >= 1 {
            return;
        }

        if attackers < 4 {
            let mut body = vec![Part::Move, Part::Move, Part::Heal];
            let max_energy = room.energy_capacity_available();
            let mut cost = 350;

            let mut isnt_potato = false;

            while cost < max_energy {
                if cost + 130 > max_energy {
                    break;
                }

                isnt_potato = true;

                body.push(Part::Attack);
                body.push(Part::Move);
                cost += 130;
            }

            if isnt_potato {
                spawn_manager.create_spawn_request(Role::Bulldozer, body, 4.0, cost, None, None);
            }
        } else if unclaimer < 1 && !unclaimer > 3 && should_spawn_unclaimer {
            let mut body = vec![Part::Move, Part::Move];
            let max_energy = room.energy_capacity_available();
            let mut cost = 100;

            let mut isnt_potato = false;

            while cost < max_energy {
                if cost + 650 > max_energy {
                    break;
                }

                isnt_potato = true;

                body.push(Part::Claim);
                body.push(Part::Move);
                cost += 650;
            }

            if isnt_potato {
                spawn_manager.create_spawn_request(Role::Unclaimer, body, 4.0, cost, None, None);
            }
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn scout(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let body = vec![Part::Move];
    let cost = get_body_cost(&body);

    let scouts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Scout)
        .unwrap_or(&Vec::new())
        .len();

    let has_observer = cache.structures.observer.is_some();

    let count = if has_observer {
        1
    } else {
        2
    };

    if scouts >= count {
        return;
    }

    spawn_manager.create_spawn_request(Role::Scout, body, 4.0, cost, None, None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn repairer(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let repairing_work_parts = cache
    .creeps
    .creeps_of_role
    .get(&Role::Repairer)
    .unwrap_or(&Vec::new())
    .iter().map(|c| game::creeps().get(c.to_string()).unwrap().body().iter().filter(|p| p.part() == Part::Work && p.hits() > 0).count() as u32)
    .sum::<u32>();

    if cache.structures.controller.as_ref().unwrap().controller.level() < 3 || cache.structures.storage.is_none() {
        return;
    }

    let repair_sites = cache.structures.needs_repair.len();

    let mut desired_repair_parts = cmp::max(repair_sites / 9, 3);

    if desired_repair_parts < 3 {
        desired_repair_parts = 3;
    }

    info!("Repairer parts: {} Desired: {} - Parts: {}", repairing_work_parts, desired_repair_parts, repair_sites);

    if repairing_work_parts >= desired_repair_parts as u32 {
        return;
    }

    let body = crate::room::spawning::creep_sizing::repairer(room, desired_repair_parts as u8, cache);
    let cost = get_body_cost(&body);

    spawn_manager.create_spawn_request(Role::Repairer, body, 55.0, cost, None, None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn builder(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let building_work_parts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Builder)
        .unwrap_or(&Vec::new())
        .iter().map(|c| game::creeps().get(c.to_string()).unwrap().body().iter().filter(|p| p.part() == Part::Work && p.hits() > 0).count() as u32)
        .sum::<u32>();

        if cache.structures.controller.as_ref().unwrap().controller.level() < 2 {
            return;
        }

    let construction_sites = cache.structures.construction_sites.len();

    let mut desired_work_parts = cmp::max(construction_sites / 3, 12);

    if desired_work_parts < 3 {
        desired_work_parts = 3;
    }

    if building_work_parts as usize >= desired_work_parts {
        return;
    }

    let body = crate::room::spawning::creep_sizing::builder(room, cache);
    let cost = get_body_cost(&body);

    spawn_manager.create_spawn_request(Role::Builder, body, 4.0, cost, None, None);

}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn upgrader(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let upgraders = cache
        .creeps
        .creeps_of_role
        .get(&Role::Upgrader)
        .unwrap_or(&Vec::new())
        .len();

    let body = crate::room::spawning::creep_sizing::upgrader(room, cache);
    let cost = get_body_cost(&body);

    if body.is_empty() {
        return;
    }

    spawn_manager.create_spawn_request(Role::Upgrader, body, 4.0, cost, None, None);
}

// TODO: Math this shit! Make it better!
//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn hauler(
    room: &Room,
    cache: &mut RoomCache,
    spawn_manager: &mut SpawnManager,
    memory: &mut ScreepsMemory,
) {
    let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

    let owning_cache = cache.rooms.get(&room.name()).unwrap();
    let mut carry_requirement = 0;

    let body = crate::room::spawning::creep_sizing::hauler(room);
    let carry_count = body.iter().filter(|p| *p == &Part::Carry).count();

    let harvester_count = owning_cache.creeps.creeps_of_role.get(&Role::Miner).unwrap_or(&Vec::new()).len();
    let remote_harvester_count = owning_cache.creeps.creeps_of_role.get(&Role::RemoteHarvester).unwrap_or(&Vec::new()).len();

    let harvester_count = harvester_count + remote_harvester_count;

    if room_memory.hauler_count == 0 || game::time() % 1002 == 0 {
        for remote in &room_memory.remotes {
            let mut scouted_data = memory.scouted_rooms.get(&remote).unwrap();

            if let Some(game_room) = game::rooms().get(*remote) {
                let mut room_cache = cache.rooms.get(&remote).unwrap();

                for source in &room_cache.resources.sources {
                    let source_ept = (source.calculate_work_parts() * 2) as usize;
                    let source = game::get_object_by_id_typed(&source.id).unwrap();

                    let (out_steps, in_steps) = if let Some(storage) = &owning_cache.structures.storage {
                        let mut out_target = MoveTarget { pos: source.pos(), range: 1 };
                        let mut in_target = MoveTarget { pos: storage.pos(), range: 1 };

                        let out_steps = out_target.find_path_to(storage.pos(), MoveOptions::default().path_age(u8::MAX)).len();
                        let in_steps = in_target.find_path_to(source.pos(), MoveOptions::default().path_age(u8::MAX)).len();

                        (out_steps, in_steps)
                    } else {
                        let spawn = room_cache.structures.spawns.values().next().unwrap();

                        let mut out_target = MoveTarget { pos: source.pos(), range: 1 };
                        let mut in_target = MoveTarget { pos: spawn.pos(), range: 1 };

                        let out_steps = out_target.find_path_to(spawn.pos(), MoveOptions::default().path_age(u8::MAX)).len();
                        let in_steps = in_target.find_path_to(source.pos(), MoveOptions::default().path_age(u8::MAX)).len();

                        (out_steps, in_steps)
                    };

                    carry_requirement += source_ept * (out_steps + in_steps);
                }
            }

        }

        for source in &owning_cache.resources.sources {
            let source_ept = (source.calculate_work_parts() * 2) as usize;
            let source = game::get_object_by_id_typed(&source.id).unwrap();
    
            let (out_steps, in_steps) = if let Some(storage) = &owning_cache.structures.storage {
                let mut out_target = MoveTarget { pos: source.pos(), range: 1 };
                let mut in_target = MoveTarget { pos: storage.pos(), range: 1 };
    
                let out_steps = out_target.find_path_to(storage.pos(), MoveOptions::default().path_age(u8::MAX)).len();
                let in_steps = in_target.find_path_to(source.pos(), MoveOptions::default().path_age(u8::MAX)).len();
    
                (out_steps, in_steps)
            } else {
                let spawn = owning_cache.structures.spawns.values().next().unwrap();
    
                let mut out_target = MoveTarget { pos: source.pos(), range: 1 };
                let mut in_target = MoveTarget { pos: spawn.pos(), range: 1 };
    
                let out_steps = out_target.find_path_to(spawn.pos(), MoveOptions::default().path_age(u8::MAX)).len();
                let in_steps = in_target.find_path_to(source.pos(), MoveOptions::default().path_age(u8::MAX)).len();
    
                (out_steps, in_steps)
            };
    
            carry_requirement += source_ept * (out_steps + in_steps);
        }

        let wanted_hauler_count = (carry_requirement as f32) / (carry_count as f32 * 50.0);

        let mut hauler_count = if wanted_hauler_count < 3.0 {
            3
        } else {
            wanted_hauler_count.round() as u32
        };

        if wanted_hauler_count > (f32::max(2.0, 15.0 / owning_cache.structures.controller.as_ref().unwrap().controller.level() as f32) * harvester_count as f32).round() {
            hauler_count = (f32::max(2.0, 15.0 / owning_cache.structures.controller.as_ref().unwrap().controller.level() as f32) * harvester_count as f32).round() as u32;
        }

        room_memory.hauler_count = hauler_count;
    }

    let wanted_count = room_memory.hauler_count;

    let hauler_count = owning_cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    if hauler_count >= wanted_count as usize && hauler_count > 3 {
        return;
    }

    let cost = get_body_cost(&body);

    let creepmem = CreepMemory {
        owning_room: room.name(),
        needs_energy: Some(true),
        ..Default::default()
    };

    let prio = if hauler_count < 3 {
        400000.0
    } else if hauler_count > (wanted_count as f32 / 2.5).ceil() as usize {
        2.4
    } else {
        4.1
    };

    spawn_manager.create_spawn_request(Role::Hauler, body, prio, cost, Some(creepmem), None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn base_hauler(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager, memory: &mut ScreepsMemory) {

    // Since it pulls from the storage, dont spawn if there is no storage.
    if cache.structures.storage.is_none() {
        return;
    }

    let thing_bc_rust_dum = Vec::new();
    let current_bh_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::BaseHauler)
        .unwrap_or(&thing_bc_rust_dum);

    let hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&thing_bc_rust_dum)
        .len();

    let required_bh_bount = match room.controller().unwrap().level() {
        1 => 0,
        2 => 1,
        3 => 1,
        4 => 1,
        5 => 1,
        6 => 1,
        7 => 1,
        8 => 1,
        _ => 1,
    };

    let max_energy = if hauler_count > 0 {
        room.energy_capacity_available()
    } else {
        room.energy_available()
    };

    let mut body = vec![Part::Move, Part::Carry];
    let mut cost = 100;

    while cost < max_energy {
        if cost + 100 > max_energy {
            break;
        }

        body.push(Part::Move);
        body.push(Part::Carry);
        cost += 100;
    }

    let should_replace = if let Some(existing_bh) = current_bh_count.iter().next() {
        let creep = game::creeps().get(existing_bh.to_string()).unwrap();

        // Existing BH time to live
        let ttl = creep.ticks_to_live().unwrap_or(0);
        // New BH time to spawn
        let tts = body.len() * 3;

        // If ttl is less than or equal to tts, replace.
        ttl <= tts as u32
    } else {
        true
    };

    if current_bh_count.len() >= required_bh_bount && !should_replace {
        return;
    }

    let creep_memory = CreepMemory {
        owning_room: room.name(),
        role: Role::BaseHauler,
        ..Default::default()
    };

    spawn_manager.create_spawn_request(Role::BaseHauler, body, 500.0, cost, Some(creep_memory), None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn fast_filler(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let fast_filler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::FastFiller)
        .unwrap_or(&Vec::new())
        .len();

    if fast_filler_count >= 2 {
        return;
    }

    let body = vec![Part::Carry, Part::Move];
    let cost = get_body_cost(&body);

    spawn_manager.create_spawn_request(Role::FastFiller, body, 50.0, cost, None, None);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn miner(room: &Room, cache: &mut CachedRoom, spawn_manager: &mut SpawnManager) {
    let miner_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Miner)
        .unwrap_or(&Vec::new())
        .len();
    let hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    for source in &cache.resources.sources {
        let parts_needed = source.parts_needed();

        if parts_needed == 0 || source.creeps.len() >= source.calculate_mining_spots(room).into() {
            continue;
        }
        let parts = crate::room::spawning::creep_sizing::miner(room, cache, parts_needed);
        let cost = get_body_cost(&parts);

        let mut priority = 6.0;

        if miner_count < hauler_count {
            priority -= 1.0;
        }
        priority += (parts_needed as f64) * 0.75;

        if miner_count < cache.resources.sources.len() {
            priority += 50.0;
        }

        let index = &cache
            .resources
            .sources
            .iter()
            .position(|s| s.id == source.id)
            .unwrap();

        let creep_memory = CreepMemory {
            owning_room: room.name(),
            task_id: Some((*index).try_into().unwrap()),
            ..Default::default()
        };

        spawn_manager.create_spawn_request(
            Role::Miner,
            parts,
            priority,
            cost,
            Some(creep_memory),
            None,
        );
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn remote_harvester(room: &Room, cache: &mut RoomCache, memory: &mut ScreepsMemory, spawn_manager: &mut SpawnManager) {
    let remotes = cache.rooms.get_mut(&room.name()).unwrap().remotes.clone();
    for remote in remotes {
        if let Some(remote_room) = game::rooms().get(remote) {
            cache.create_if_not_exists(&remote_room, memory, None);
            let cache = cache.rooms.get_mut(&remote_room.name()).unwrap();

            for source in cache.resources.sources.iter() {
                let parts_needed = source.parts_needed();

                if parts_needed == 0 || source.creeps.len() >= source.calculate_mining_spots(&remote_room).into() {
                    continue;
                }

                let parts = crate::room::spawning::creep_sizing::miner(&room, cache, parts_needed);
                let cost = get_body_cost(&parts);

                let mut priority = 0.0;

                priority += (parts_needed as f64) * 1.5;

                let index = &cache
                    .resources
                    .sources
                    .iter()
                    .position(|s| s.id == source.id)
                    .unwrap();

                let creep_memory = CreepMemory {
                    owning_room: room.name(),
                    owning_remote: Some(remote),
                    task_id: Some((*index).try_into().unwrap()),
                    ..Default::default()
                };

                spawn_manager.create_spawn_request(
                    Role::RemoteHarvester,
                    parts,
                    priority,
                    cost,
                    Some(creep_memory),
                    None,
                );
            }
        } else {
            let cached_room = cache.rooms.get_mut(&room.name()).unwrap();
            let existing_physical_observers = cached_room
                .creeps
                .creeps_of_role
                .get(&Role::PhysicalObserver);

            if let Some(existing_observers) = existing_physical_observers {
                for existing_observer in existing_observers.iter() {
                    let creep = game::creeps().get(existing_observer.to_string()).unwrap();
                    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

                    if creep_memory.scout_target.is_none() || creep.room().unwrap().name() == creep_memory.scout_target.unwrap() {
                        creep_memory.scout_target = Some(remote);

                        return;
                    }
                }
            } else {

                let body = vec![Part::Move];
                let cost = get_body_cost(&body);

                let creep_memory = CreepMemory {
                    owning_room: room.name(),
                    scout_target: Some(remote),
                    ..Default::default()
                };

                spawn_manager.create_spawn_request(Role::PhysicalObserver, body, 10.0, cost, Some(creep_memory), None);
            }
        }
    }
}