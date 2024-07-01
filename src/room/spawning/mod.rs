use std::{cmp, collections::HashMap};

use creep_sizing::base_hauler_body;
use log::info;
use screeps::{find, game, HasPosition, Part, ResourceType, Room, SharedCreepProperties};
use spawn_manager::{SpawnManager, SpawnRequest};
use strum::IntoEnumIterator;

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory}, movement::move_target::{MoveOptions, MoveTarget}, utils::get_body_cost
};

use super::cache::tick_cache::{CachedRoom, RoomCache};

pub mod creep_sizing;
pub mod spawn_manager;

pub fn get_required_role_counts(room_cache: &CachedRoom) -> HashMap<Role, u32> {
    let mut map = HashMap::new();

    let controller = &room_cache.structures.controller.as_ref().unwrap().controller;

    for role in Role::iter() {
        let score = match role {
            Role::Harvester => 1,
            Role::Hauler => 1,
            Role::Repairer => { if controller.level() > 2 { 1 } else { 0 } },
            Role::BaseHauler => { if room_cache.structures.storage.is_some() { 1 } else { 0 } },
            Role::FastFiller => { if room_cache.structures.containers.fast_filler.is_some() { 2 } else { 0 } },
            Role::Upgrader => { if controller.ticks_to_downgrade() < Some(1500) { 1 } else { 0 } },
            Role::Scout => { if controller.level() > 2 { 1 } else { 0 } },
            _ => 0,
        };

        if score != 0 {
            map.insert(role, score);
        }
    }

    map
}

// TODO:
//  Add required role counts
//  Fuck this shit man, this looks like ass
//  Tweak a shit load of numbers. Spawning needs to be PERFECT.
//  TODO!!! Fix the double remote spawning bug!! This is BAD...

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn create_spawn_requests_for_room(room: &Room, cache: &mut RoomCache, memory: &mut ScreepsMemory) -> Vec<SpawnRequest> {
    let room_cache = cache.rooms.get(&room.name()).unwrap();

    let requests = vec![
        miner(room, room_cache, &mut cache.spawning),
        base_hauler(room, room_cache, &mut cache.spawning),
        fast_filler(room, room_cache, &mut cache.spawning),
        flag_attacker(room, room_cache, &mut cache.spawning),
        builder(room, room_cache, &mut cache.spawning),
        repairer(room, room_cache, &mut cache.spawning),
        upgrader(room, room_cache, &mut cache.spawning),
        scout(room, room_cache, &mut cache.spawning),

        // More inter-room creeps that require the WHOLE cache.
        remote_harvester(room, cache, memory),
        hauler(room, cache, memory),
    ];

    requests.into_iter().flatten().collect()
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn flag_attacker(room: &Room, cache: &CachedRoom, spawn_manager: &mut SpawnManager) -> Option<SpawnRequest> {
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
            return None;
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
                return Some(spawn_manager.create_room_spawn_request(Role::Bulldozer, body, 4.0, cost, room.name(), None, None, None));
            }
            return None;
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
                return Some(spawn_manager.create_room_spawn_request(Role::Unclaimer, body, 4.5, cost, room.name(), None, None, None));
            }
            return None;
        }
    }
    None
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn scout(room: &Room, cache: &CachedRoom, spawn_manager: &mut SpawnManager) -> Option<SpawnRequest> {
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
        return None;
    }

    // These guys are SUPER cheap, but SUPER important.
    Some(spawn_manager.create_room_spawn_request(Role::Scout, body, 400000.0, cost, room.name(), None, None, None))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn repairer(room: &Room, cache: &CachedRoom, spawn_manager: &mut SpawnManager) -> Option<SpawnRequest> {
    let repairing_work_parts = cache
    .creeps
    .creeps_of_role
    .get(&Role::Repairer)
    .unwrap_or(&Vec::new())
    .iter().map(|c| game::creeps().get(c.to_string()).unwrap().body().iter().filter(|p| p.part() == Part::Work && p.hits() > 0).count() as u32)
    .sum::<u32>();

    if cache.structures.controller.as_ref().unwrap().controller.level() < 3 || cache.structures.storage.is_none() || cache.structures.storage.as_ref().unwrap().store().get_used_capacity(Some(ResourceType::Energy)) < 10000 {
        return None;
    }

    let repair_sites = cache.structures.needs_repair.len();

    let mut desired_repair_parts = cmp::max(repair_sites / 9, 3);

    if desired_repair_parts < 3 {
        desired_repair_parts = 3;
    }

    info!("Repairer parts: {} Desired: {} - Parts: {}", repairing_work_parts, desired_repair_parts, repair_sites);

    if repairing_work_parts >= desired_repair_parts as u32 {
        return None;
    }

    let body = crate::room::spawning::creep_sizing::repairer_body(room, desired_repair_parts as u8, cache);
    let cost = get_body_cost(&body);

    Some(spawn_manager.create_room_spawn_request(Role::Repairer, body, 55.0, cost, room.name(), None, None, None))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn builder(room: &Room, cache: &CachedRoom, spawn_manager: &mut SpawnManager) -> Option<SpawnRequest> {
    let building_work_parts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Builder)
        .unwrap_or(&Vec::new())
        .iter().map(|c| game::creeps().get(c.to_string()).unwrap().body().iter().filter(|p| p.part() == Part::Work && p.hits() > 0).count() as u32)
        .sum::<u32>();

    if cache.structures.controller.as_ref().unwrap().controller.level() < 2 {
        return None;
    }

    if cache.structures.storage.is_some() && cache.structures.storage.as_ref().unwrap().store().get_used_capacity(Some(ResourceType::Energy)) < 10000 {
        return None;
    }

    let construction_sites = cache.structures.construction_sites.len();

    let desired_work_parts = (construction_sites as f32 / 3.0).round().clamp(3.0, 12.0);

    if building_work_parts as f32 >= desired_work_parts {
        return None;
    }

    let body = crate::room::spawning::creep_sizing::builder_body(room, cache);
    let cost = get_body_cost(&body);

    let priority = desired_work_parts as f64 * 0.75;

    Some(spawn_manager.create_room_spawn_request(Role::Builder, body, 4.5, cost, room.name(), None, None, None))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn upgrader(room: &Room, cache: &CachedRoom, spawn_manager: &mut SpawnManager) -> Option<SpawnRequest> {
    let body = crate::room::spawning::creep_sizing::upgrader_body(room, cache);
    let cost = get_body_cost(&body);

    if body.is_empty() {
        return None;
    }

    Some(spawn_manager.create_room_spawn_request(Role::Upgrader, body, 4.0, cost, room.name(), None, None, None))
}

// TODO: Math this shit! Make it better!
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn hauler(
    room: &Room,
    cache: &RoomCache,
    memory: &mut ScreepsMemory,
) -> Option<SpawnRequest> {
    let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

    let owning_cache = cache.rooms.get(&room.name()).unwrap();
    let mut carry_requirement: u128 = 0;

    let body = crate::room::spawning::creep_sizing::hauler_body(room);
    let carry_count = body.iter().filter(|p| *p == &Part::Carry).count();

    let harvester_count = owning_cache.creeps.creeps_of_role.get(&Role::Harvester).unwrap_or(&Vec::new()).len();
    let remote_harvester_count = owning_cache.creeps.creeps_of_role.get(&Role::RemoteHarvester).unwrap_or(&Vec::new()).len();

    let harvester_count = harvester_count + remote_harvester_count;

    if room_memory.hauler_count == 0 || game::time() % 100 == 0 || room_memory.hauler_count > 200 {
        for remote in &room_memory.remotes {

            if game::rooms().get(*remote).is_some() {
                let room_cache = cache.rooms.get(remote).unwrap();

                for source in &room_cache.resources.sources {
                    let source_ept = (source.calculate_work_parts() * 2) as u128;
                    let source = game::get_object_by_id_typed(&source.id).unwrap();

                    let (out_steps, in_steps) = if let Some(storage) = &owning_cache.structures.storage {
                        let mut out_target = MoveTarget { pos: source.pos(), range: 1 };
                        let mut in_target = MoveTarget { pos: storage.pos(), range: 1 };

                        let out_steps = out_target.find_path_to(storage.pos(), MoveOptions::default().path_age(u8::MAX)).len() as u128;
                        let in_steps = in_target.find_path_to(source.pos(), MoveOptions::default().path_age(u8::MAX)).len() as u128;

                        (out_steps, in_steps)
                    } else {
                        let spawn = owning_cache.structures.spawns.values().next().unwrap();

                        let mut out_target = MoveTarget { pos: source.pos(), range: 1 };
                        let mut in_target = MoveTarget { pos: spawn.pos(), range: 1 };

                        let out_steps = out_target.find_path_to(spawn.pos(), MoveOptions::default().path_age(u8::MAX)).len() as u128;
                        let in_steps = in_target.find_path_to(source.pos(), MoveOptions::default().path_age(u8::MAX)).len() as u128;

                        (out_steps, in_steps)
                    };

                    carry_requirement += source_ept * (out_steps + in_steps);
                }
            }

        }

        for source in &owning_cache.resources.sources {
            let source_ept = (source.calculate_work_parts() * 2) as u128;
            let source = game::get_object_by_id_typed(&source.id).unwrap();
    
            let (out_steps, in_steps) = if let Some(storage) = &owning_cache.structures.storage {
                let mut out_target = MoveTarget { pos: source.pos(), range: 1 };
                let mut in_target = MoveTarget { pos: storage.pos(), range: 1 };
    
                let out_steps = out_target.find_path_to(storage.pos(), MoveOptions::default().path_age(u8::MAX)).len() as u128;
                let in_steps = in_target.find_path_to(source.pos(), MoveOptions::default().path_age(u8::MAX)).len() as u128;
    
                (out_steps, in_steps)
            } else {
                let spawn = owning_cache.structures.spawns.values().next().unwrap();
    
                let mut out_target = MoveTarget { pos: source.pos(), range: 1 };
                let mut in_target = MoveTarget { pos: spawn.pos(), range: 1 };
    
                let out_steps = out_target.find_path_to(spawn.pos(), MoveOptions::default().path_age(u8::MAX)).len() as u128;
                let in_steps = in_target.find_path_to(source.pos(), MoveOptions::default().path_age(u8::MAX)).len() as u128;
    
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

        //if wanted_hauler_count > (f32::max(2.0, 15.0 / owning_cache.structures.controller.as_ref().unwrap().controller.level() as f32) * harvester_count as f32).round() {
        //    hauler_count = (f32::max(2.0, 15.0 / owning_cache.structures.controller.as_ref().unwrap().controller.level() as f32) * harvester_count as f32).round() as u32;
        //}

        let clamped = hauler_count.clamp(3, 200);
        room_memory.hauler_count = clamped;
    }

    let wanted_count = room_memory.hauler_count;

    let hauler_count = owning_cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    if hauler_count >= wanted_count as usize && hauler_count > 3 {
        return None;
    }

    if harvester_count == 0 {
        return None;
    }

    let cost = get_body_cost(&body);

    let creepmem = CreepMemory {
        owning_room: room.name(),
        needs_energy: Some(true),
        ..Default::default()
    };

    // If we have less than 3 total haulers.
    let prio = if hauler_count < 3 {
        400000.0
    // If we have more than half of the wanted count.
    } else if hauler_count > (wanted_count as f32 / 2.0).ceil() as usize {
        2.0

    // If we are at a third of the hauler count
    } else if hauler_count > (wanted_count as f32 / 3.0).ceil() as usize {
        3.5
    } else {
        // TODO
        // I might need to tweak this number a bit.
        4.0
    };

    Some(cache.spawning.create_room_spawn_request(Role::Hauler, body, prio, cost, room.name(), Some(creepmem), None, None))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn base_hauler(room: &Room, cache: &CachedRoom, spawn_manager: &mut SpawnManager) -> Option<SpawnRequest> {

    // Since it pulls from the storage, dont spawn if there is no storage.
    cache.structures.storage.as_ref()?;

    let thing_bc_rust_dum = Vec::new();
    let current_bh_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::BaseHauler)
        .unwrap_or(&thing_bc_rust_dum);

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

    let body = base_hauler_body(room, cache);
    let cost = get_body_cost(&body);

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
        return None;
    }

    let creep_memory = CreepMemory {
        owning_room: room.name(),
        role: Role::BaseHauler,
        ..Default::default()
    };

    let req = spawn_manager.create_room_spawn_request(Role::BaseHauler, body, f64::MAX, cost, room.name(), Some(creep_memory), None, None);

    Some(req)
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn fast_filler(room: &Room, cache: &CachedRoom, spawn_manager: &mut SpawnManager) -> Option<SpawnRequest> {
    let fast_filler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::FastFiller)
        .unwrap_or(&Vec::new())
        .len();

    if fast_filler_count >= 2 {
        return None;
    }

    let body = vec![Part::Carry, Part::Move];
    let cost = get_body_cost(&body);

    Some(spawn_manager.create_room_spawn_request(Role::FastFiller, body, f64::MAX, cost, room.name(), None, None, None))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn miner(room: &Room, cache: &CachedRoom, spawn_manager: &mut SpawnManager) -> Option<SpawnRequest> {
    let miner_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Harvester)
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
        let parts = crate::room::spawning::creep_sizing::miner_body(room, cache, parts_needed);
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

        let req = spawn_manager.create_room_spawn_request(
            Role::Harvester,
            parts,
            priority,
            cost,
            room.name(),
            Some(creep_memory),
            None,
            None
        );

        return Some(req);
    }

    None
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn remote_harvester(room: &Room, cache: &RoomCache, memory: &mut ScreepsMemory) -> Option<SpawnRequest> {
    let remotes = cache.rooms.get(&room.name()).unwrap().remotes.clone();

    let cached = cache.rooms.get(&room.name()).unwrap();

    let harvester_count = cached
        .creeps
        .creeps_of_role
        .get(&Role::RemoteHarvester)
        .unwrap_or(&Vec::new())
        .len();

    let hauler_count = cached
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    for remote in remotes {
        if let Some(remote_room) = game::rooms().get(remote) {
            if let Some(remote_memory) = memory.remote_rooms.get(&remote) {
                if remote_memory.under_attack {
                    continue;
                }
            }
            let room_cache = cache.rooms.get(&remote_room.name()).unwrap();

            for source in room_cache.resources.sources.iter() {
                let parts_needed = source.parts_needed();

                if parts_needed == 0 || source.creeps.len() >= source.calculate_mining_spots(&remote_room).into() {
                    continue;
                }

                let parts = crate::room::spawning::creep_sizing::miner_body(room, room_cache, parts_needed);
                let cost = get_body_cost(&parts);

                let mut priority = 0.0;

                priority += (parts_needed as f64) * 1.5;

                let index = &room_cache
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

                if hauler_count < harvester_count {
                    priority -= 10.0;
                }

                let req = cache.spawning.create_room_spawn_request(
                    Role::RemoteHarvester,
                    parts,
                    priority,
                    cost,
                    room.name(),
                    Some(creep_memory),
                    None,
                    None
                );

                return Some(req);
            }
        } else {
            let cached_room = cache.rooms.get(&room.name()).unwrap();
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

                        return None;
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

                let req = cache.spawning.create_room_spawn_request(Role::PhysicalObserver, body, 10.0, cost, room.name(), Some(creep_memory), None, None);
                return Some(req);
            }
        }
    }
    None
}

// --- Combat dependent things. Called globally, not per room.

pub fn spawn_claimer(cache: &mut RoomCache, memory: &mut ScreepsMemory, spawn_manager: &mut SpawnManager) {

}