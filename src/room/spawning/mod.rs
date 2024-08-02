use std::{cmp, collections::HashMap, vec};

use creep_sizing::{base_hauler_body, storage_sitter_body};
use log::info;
use screeps::{find, game, HasId, Part, ResourceType, Room, SharedCreepProperties};
use spawn_manager::{SpawnManager, SpawnRequest};
use strum::IntoEnumIterator;

use crate::{
    formation::duo::{self, duo_utils}, memory::{CreepMemory, DuoMemory, Role, ScoutedSource, ScreepsMemory}, utils::{self, get_body_cost, get_unique_id, role_to_name}
};

use super::{
    cache::tick_cache::{CachedRoom, RoomCache},
    planning::room::construction,
};

pub mod creep_sizing;
pub mod spawn_manager;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_required_role_counts(room_cache: &CachedRoom) -> HashMap<Role, u32> {
    let mut map = HashMap::new();

    let controller = &room_cache
        .structures
        .controller
        .as_ref()
        .unwrap()
        .controller;

    let harvester_count = room_cache
        .creeps
        .creeps_of_role
        .get(&Role::Harvester)
        .unwrap_or(&Vec::new())
        .len();

    for role in Role::iter() {
        let score = match role {
            Role::Harvester => 1,
            Role::Hauler => 3,
            Role::Builder => {
                let mut storage_blocked = false;

                if let Some(storage) = &room_cache.structures.storage {
                    if storage
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy))
                        < 10000
                    {
                        storage_blocked = true;
                    }
                }

                if controller.level() >= 2
                    && !room_cache.structures.construction_sites.is_empty()
                    && harvester_count >= 1
                    && !storage_blocked
                {
                    1
                } else {
                    0
                }
            }
            Role::Repairer => {
                let mut storage_blocked = false;

                if let Some(storage) = &room_cache.structures.storage {
                    if storage
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy))
                        < 10000
                    {
                        storage_blocked = true;
                    }
                }

                if controller.level() > 2 && !storage_blocked && harvester_count >= 1 {
                    1
                } else {
                    0
                }
            }
            Role::StorageSitter => {
                if room_cache.rcl >= 5 {
                    1
                } else {
                    0
                }
            }
            Role::BaseHauler => {
                if room_cache.structures.storage.is_some() && harvester_count >= 1 {
                    1
                } else {
                    0
                }
            }
            Role::FastFiller => {
                if room_cache.structures.containers.fast_filler.is_some()
                    && !room_cache
                        .structures
                        .containers
                        .fast_filler
                        .as_ref()
                        .unwrap()
                        .is_empty()
                    && harvester_count >= 1
                {
                    2
                } else {
                    0
                }
            }
            Role::Upgrader => {
                let mut storage_blocked = false;

                if let Some(storage) = &room_cache.structures.storage {
                    if storage
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy))
                        < 10000
                    {
                        storage_blocked = true;
                    }
                }

                if controller.level() < 8
                    || (controller.ticks_to_downgrade() < Some(1500) && controller.level() >= 8)
                        && harvester_count >= 1
                        && !storage_blocked
                {
                    1
                } else {
                    0
                }
            }
            Role::Scout => {
                if controller.level() > 2 {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        };

        if score != 0 {
            map.insert(role, score);
        }
    }

    map
}


pub fn temp_duo_spawning(
    room: &Room,
    cache: &RoomCache,
    memory: &mut ScreepsMemory,
) -> Option<SpawnRequest> {
    if memory.formations.duos.is_empty() {
        memory.formations.duos.insert(utils::get_unique_id(), DuoMemory::default());
    }

    for (formation_id, duo_memory) in &memory.formations.duos.clone() {
        let creeps = duo_memory.creeps.clone();
        let mut gcreeps = Vec::new();

        for creep in creeps {
            let gcreep = game::creeps().get(creep.to_string());

            if let Some(gcreep) = gcreep {
                gcreeps.push(gcreep);
            } else {
                memory.formations.duos.get_mut(formation_id).unwrap().creeps.retain(|x| *x != creep);
            }
        }

        if duo_utils::get_attacker(&gcreeps).is_none() {
            let mut body = vec![Part::Move, Part::Move, Part::Attack];
            let cost = get_body_cost(&body);

            let creep_name = format!("{}-{}-{}", role_to_name(Role::InvaderDuoAttacker), room.name(), get_unique_id());

            let creep_memory = CreepMemory {
                role: Role::InvaderDuoAttacker,
                owning_room: room.name(),
                //target_room: Some("W1N1".to_string()),
                ..Default::default()
            };

            let req = cache.spawning.create_room_spawn_request(Role::InvaderDuoAttacker, body, 4.0, cost, room.name(), Some(creep_memory), None, Some(creep_name.clone()));
            memory.formations.duos.get_mut(formation_id).unwrap().creeps.push(creep_name);

            return Some(req);
        }

        if duo_utils::get_healer(&gcreeps).is_none() {
            let mut body = vec![Part::Move, Part::Move, Part::Heal];
            let cost = get_body_cost(&body);

            let creep_name = format!("{}-{}-{}", role_to_name(Role::InvaderDuoHealer), room.name(), get_unique_id());

            let creep_memory = CreepMemory {
                role: Role::InvaderDuoHealer,
                owning_room: room.name(),
                //target_room: Some("W1N1".to_string()),
                ..Default::default()
            };

            let req = cache.spawning.create_room_spawn_request(Role::InvaderDuoHealer, body, 4.0, cost, room.name(), Some(creep_memory), None, Some(creep_name.clone()));
            memory.formations.duos.get_mut(formation_id).unwrap().creeps.push(creep_name);

            return Some(req);
        }
    }

    None
}
// TODO:
//  Add required role counts
//  Fuck this shit man, this looks like ass
//  Tweak a shit load of numbers. Spawning needs to be PERFECT.
//  TODO!!! Which asshole wrote this shit, god they suck at programming.

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn create_spawn_requests_for_room(
    room: &Room,
    cache: &mut RoomCache,
    memory: &mut ScreepsMemory,
) -> Vec<SpawnRequest> {
    let room_cache = cache.rooms.get(&room.name()).unwrap();

    let requests = vec![
        harvester(room, room_cache, &mut cache.spawning),
        base_hauler(room, room_cache, &mut cache.spawning),
        storage_sitter(room, room_cache, &mut cache.spawning),
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
pub fn flag_attacker(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
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
        if let Some(target_room) = flag.room() {
            if target_room.find(find::HOSTILE_CREEPS, None).is_empty()
                && target_room.find(find::HOSTILE_SPAWNS, None).is_empty()
                && target_room.find(find::HOSTILE_STRUCTURES, None).is_empty()
            {
                let range = utils::calc_room_distance(&target_room.name(), &room.name(), true);

                if range > 600/50 {
                    should_spawn_unclaimer = true;
                }
            }
        }

        if attackers >= 4 && unclaimer >= 1 {
            return None;
        }

        if attackers < 4 {
            if cache.rcl >= 8 {
                let mut body = Vec::new();

                for _ in 0..12 {
                    body.push(Part::Attack);
                }

                for _ in 0..25 {
                    body.push(Part::Move);
                }

                for _ in 0..13 {
                    body.push(Part::Heal);
                }

                let cost = get_body_cost(&body);

                return Some(spawn_manager.create_room_spawn_request(
                    Role::Bulldozer,
                    body,
                    4.0,
                    cost,
                    room.name(),
                    None,
                    None,
                    None,
                ));
            } else {
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
                    return Some(spawn_manager.create_room_spawn_request(
                        Role::Bulldozer,
                        body,
                        4.0,
                        cost,
                        room.name(),
                        None,
                        None,
                        None,
                    ));
                }
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
                return Some(spawn_manager.create_room_spawn_request(
                    Role::Unclaimer,
                    body,
                    12.0,
                    cost,
                    room.name(),
                    None,
                    None,
                    None,
                ));
            }
            return None;
        }
    }
    None
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn scout(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
    let body = vec![Part::Move];
    let cost = get_body_cost(&body);

    let scouts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Scout)
        .unwrap_or(&Vec::new())
        .len();

    let has_observer = cache.structures.observer.is_some();

    let count = if has_observer { 1 } else { 2 };

    if scouts >= count {
        return None;
    }

    // These guys are SUPER cheap, but SUPER important.
    Some(spawn_manager.create_room_spawn_request(
        Role::Scout,
        body,
        400000.0,
        cost,
        room.name(),
        None,
        None,
        None,
    ))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn repairer(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
    let repairing_work_parts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Repairer)
        .unwrap_or(&Vec::new())
        .iter()
        .map(|c| {
            game::creeps()
                .get(c.to_string())
                .unwrap()
                .body()
                .iter()
                .filter(|p| p.part() == Part::Work && p.hits() > 0)
                .count() as u32
        })
        .sum::<u32>();

    if (cache
        .structures
        .controller
        .as_ref()
        .unwrap()
        .controller
        .level()
        < 3
        || cache.structures.storage.is_none()
        || (cache
            .structures
            .storage
            .as_ref()
            .unwrap()
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            < 10000
            && repairing_work_parts >= 1))
        && repairing_work_parts >= 1
    {
        return None;
    }

    if let Some(storage) = &cache.structures.storage {
        if storage
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            < 10000
        {
            return None;
        }
    }

    let repair_sites = cache.structures.needs_repair.len();

    if repair_sites == 0 {
        return None;
    }

    let mut desired_repair_parts = cmp::max(repair_sites / 9, 3);

    if desired_repair_parts < 3 {
        desired_repair_parts = 3;
    }

    if repairing_work_parts >= desired_repair_parts as u32 {
        return None;
    }

    let body =
        crate::room::spawning::creep_sizing::repairer_body(room, desired_repair_parts as u8, cache);
    let cost = get_body_cost(&body);

    Some(spawn_manager.create_room_spawn_request(
        Role::Repairer,
        body,
        4.0,
        cost,
        room.name(),
        None,
        None,
        None,
    ))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn builder(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
    let building_work_parts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Builder)
        .unwrap_or(&Vec::new())
        .iter()
        .map(|c| {
            game::creeps()
                .get(c.to_string())
                .unwrap()
                .body()
                .iter()
                .filter(|p| p.part() == Part::Work && p.hits() > 0)
                .count() as u32
        })
        .sum::<u32>();
    let room_level = cache
        .structures
        .controller
        .as_ref()
        .unwrap()
        .controller
        .level();

    if room_level < 2 {
        return None;
    }

    if room_level >= 8 && cache.structures.construction_sites.is_empty() {
        return None;
    }

    if cache.structures.storage.is_some()
        && (cache
            .structures
            .storage
            .as_ref()
            .unwrap()
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            < 10000
            && building_work_parts >= 1)
    {
        return None;
    }

    let construction_sites = cache.structures.construction_sites.len();

    let desired_work_parts = (construction_sites as f32 * 1.5).round().clamp(3.0, 20.0);

    if building_work_parts as f32 >= desired_work_parts || construction_sites == 0 {
        return None;
    }

    let body = crate::room::spawning::creep_sizing::builder_body(room, cache);
    let cost = get_body_cost(&body);

    if !body.contains(&Part::Work) {
        return None;
    }

    Some(spawn_manager.create_room_spawn_request(
        Role::Builder,
        body,
        4.5,
        cost,
        room.name(),
        None,
        None,
        None,
    ))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn upgrader(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
    let body = crate::room::spawning::creep_sizing::upgrader_body(room, cache);
    let cost = get_body_cost(&body);

    if body.is_empty() {
        return None;
    }

    let controller = &cache.structures.controller.as_ref().unwrap().controller;

    let room_cache = false;
    if let Some(storage) = &cache.structures.storage {
        if storage
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            < 22000
            && controller.ticks_to_downgrade() > Some(5000)
        {
            return None;
        }
    }

    // Dont need em if we are level 8 and have a lot of ticks to downgrade.
    if controller.level() == 8 && controller.ticks_to_downgrade() > Some(120000) {
        return None;
    }

    let mut priority = 4.0;
    priority += (body.len() as f64 / 3.0).round();

    Some(spawn_manager.create_room_spawn_request(
        Role::Upgrader,
        body,
        priority,
        cost,
        room.name(),
        None,
        None,
        None,
    ))
}

// TODO: Math this shit! Make it better!
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn hauler(room: &Room, cache: &RoomCache, memory: &mut ScreepsMemory) -> Option<SpawnRequest> {
    let room_memory = memory.rooms.get(&room.name()).unwrap().clone();
    let owning_cache = cache.rooms.get(&room.name()).unwrap();

    let harvester_count = owning_cache
        .creeps
        .creeps_of_role
        .get(&Role::Harvester)
        .unwrap_or(&Vec::new())
        .len();
    let remote_harvester_count = owning_cache
        .creeps
        .creeps_of_role
        .get(&Role::RemoteHarvester)
        .unwrap_or(&Vec::new())
        .len();

    let harvester_count = harvester_count + remote_harvester_count;

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

    let body = crate::room::spawning::creep_sizing::hauler_body(
        room,
        cache.rooms.get(&room.name()).unwrap(),
    );
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
    } else if hauler_count < (wanted_count as f32 / 2.0).ceil() as usize {
        5.0

    // If we are at a third of the hauler count
    } else if hauler_count < (wanted_count as f32 / 3.0).ceil() as usize {
        10.0
    } else {
        // TODO
        // I might need to tweak this number a bit.
        4.0
    };

    // TODO
    // Patchwork fix to stop idle haulers from clogging space.
    // But hey, it works, somewhat.
    if owning_cache.idle_haulers >= 3 {
        return None;
    }

    Some(cache.spawning.create_room_spawn_request(
        Role::Hauler,
        body,
        prio,
        cost,
        room.name(),
        Some(creepmem),
        None,
        None,
    ))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn base_hauler(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
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
        if current_bh_count.len() >= required_bh_bount {
            return None;
        }

        let creep = game::creeps().get(existing_bh.to_string()).unwrap();

        // Existing BH time to live
        let ttl = creep.ticks_to_live().unwrap_or(u32::MAX);
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

    let priority = if current_bh_count.len() < (required_bh_bount / 2) {
        f64::MAX
    } else {
        4.0
    };

    let creep_memory = CreepMemory {
        owning_room: room.name(),
        role: Role::BaseHauler,
        ..Default::default()
    };

    let req = spawn_manager.create_room_spawn_request(
        Role::BaseHauler,
        body,
        priority,
        cost,
        room.name(),
        Some(creep_memory),
        None,
        None,
    );

    Some(req)
}

pub fn storage_sitter(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
    if cache.rcl < 5 {
        return None;
    }

    let body = storage_sitter_body(room, cache);
    let cost = get_body_cost(&body);

    let fuck_rust = &Vec::new();
    let sitter_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::StorageSitter)
        .unwrap_or(fuck_rust);

    let should_replace = if let Some(current_ss) = sitter_count.iter().next() {
        if sitter_count.len() > 1 {
            return None;
        }

        let creep = game::creeps().get(current_ss.to_string()).unwrap();

        let ttl = creep.ticks_to_live().unwrap_or(u32::MAX);
        let spawn_time = body.len() * 3;

        ttl < spawn_time as u32
    } else {
        true
    };

    if !sitter_count.is_empty() && !should_replace {
        return None;
    }

    let creep_memory = CreepMemory {
        owning_room: room.name(),
        role: Role::StorageSitter,
        ..Default::default()
    };

    Some(spawn_manager.create_room_spawn_request(
        Role::StorageSitter,
        body,
        40.0,
        cost,
        room.name(),
        Some(creep_memory),
        None,
        None,
    ))
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn fast_filler(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
    let fast_filler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::FastFiller)
        .unwrap_or(&Vec::new())
        .len();

    let harvester_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Harvester)
        .unwrap_or(&Vec::new())
        .len();

    if fast_filler_count >= 2 {
        return None;
    }

    let level = cache
        .structures
        .controller
        .as_ref()
        .unwrap()
        .controller
        .level();
    if cache.structures.containers.fast_filler.is_none()
        && cache.structures.links.fast_filler.is_none()
    {
        return None;
    }

    let body = if level < 7 {
        vec![Part::Carry, Part::Move]
    } else if level == 7 {
        vec![Part::Carry, Part::Carry, Part::Move]
    } else {
        vec![
            Part::Carry,
            Part::Carry,
            Part::Carry,
            Part::Carry,
            Part::Move,
        ]
    };

    if harvester_count == 0 {
        return None;
    }

    let cost = get_body_cost(&body);

    Some(spawn_manager.create_room_spawn_request(
        Role::FastFiller,
        body,
        f64::MAX,
        cost,
        room.name(),
        None,
        None,
        None,
    ))
}

// TODO: rewrite this, its a mess.
// Specifically, the harvester and remote harvester functions.

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn harvester(
    room: &Room,
    cache: &CachedRoom,
    spawn_manager: &mut SpawnManager,
) -> Option<SpawnRequest> {
    let harvester_count = cache
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
        let max_parts_for_source = source.max_parts_needed();
        let current_parts_on_source = source.calculate_work_parts(cache);
        let parts_needed_on_source = source.parts_needed(cache);

        let current_creeps_on_source = source.creeps.len();
        let max_mining_positions = source.calculate_mining_spots(room);

        let source_index = &cache
            .resources
            .sources
            .iter()
            .position(|s| s.source.id() == source.source.id())
            .unwrap();

        // If we have enough parts on the source, just skip it.
        if current_parts_on_source >= max_parts_for_source
            || current_creeps_on_source >= max_mining_positions as usize
        {
            continue;
        }

        let (filled, body) = creep_sizing::miner_body(room, cache, parts_needed_on_source, false);
        let cost = get_body_cost(&body);

        // We have a creep here, so its mining.
        // Therefore, we can build the biggest one to try and replace it
        // with a bigger one. CPU isnt cheap. This shit cost me $130.
        if !source.creeps.is_empty() {
            let (filled, body) = creep_sizing::miner_body(room, cache, max_parts_for_source, true);
            let cost = get_body_cost(&body);

            let mut priority = 4.0 * parts_needed_on_source as f64;

            return Some(spawn_manager.create_room_spawn_request(
                Role::Harvester,
                body,
                priority,
                cost,
                room.name(),
                Some(CreepMemory {
                    owning_room: room.name(),
                    task_id: Some(*source_index as u128),
                    ..Default::default()
                }),
                None,
                None,
            ));
        }

        let mut priority = 4.0;

        if harvester_count < hauler_count {
            priority *= 5.0;
        }

        if current_creeps_on_source == 0 {
            priority = 400000.0;
        }

        priority += parts_needed_on_source as f64;

        return Some(spawn_manager.create_room_spawn_request(
            Role::Harvester,
            body,
            priority,
            cost,
            room.name(),
            Some(CreepMemory {
                owning_room: room.name(),
                task_id: Some(*source_index as u128),
                ..Default::default()
            }),
            None,
            None,
        ));
    }

    None
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn remote_harvester(
    room: &Room,
    cache: &RoomCache,
    memory: &mut ScreepsMemory,
) -> Option<SpawnRequest> {
    let owning_room_memory = memory.rooms.get(&room.name()).unwrap();
    let owning_cache = cache.rooms.get(&room.name()).unwrap();

    let harvester_count = owning_cache
        .creeps
        .creeps_of_role
        .get(&Role::Harvester)
        .unwrap_or(&Vec::new())
        .len();

    let hauler_count = owning_cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    for remote_name in &owning_room_memory.remotes {
        if let Some(remote_cache) = cache.rooms.get(remote_name) {
            for source in &remote_cache.resources.sources {
                let max_parts_for_source = source.max_parts_needed();
                let parts_needed_on_source = source.parts_needed(remote_cache);

                if parts_needed_on_source == 0 {
                    continue;
                }

                let current_creeps_on_source = source.creeps.len();
                let max_mining_positions =
                    source.calculate_mining_spots(&game::rooms().get(*remote_name).unwrap());

                let source_index = &remote_cache
                    .resources
                    .sources
                    .iter()
                    .position(|s| s.source.id() == source.source.id())
                    .unwrap();

                // If we have enough parts on the source, just skip it.
                if parts_needed_on_source == 0
                    || current_creeps_on_source >= max_mining_positions as usize
                {
                    continue;
                }

                let (filled, body) =
                    creep_sizing::miner_body(room, remote_cache, parts_needed_on_source, false);
                let cost = get_body_cost(&body);

                // We have a creep here, so its mining.
                // Therefore, we can build the biggest one to try and replace it
                // with a bigger one. CPU isnt cheap. This shit cost me $130.
                if !source.creeps.is_empty() {
                    let (filled, body) =
                        creep_sizing::miner_body(room, remote_cache, max_parts_for_source, true);
                    let cost = get_body_cost(&body);

                    let mut priority = 4.0 * parts_needed_on_source as f64;

                    return Some(cache.spawning.create_room_spawn_request(
                        Role::RemoteHarvester,
                        body,
                        priority,
                        cost,
                        room.name(),
                        Some(CreepMemory {
                            owning_room: room.name(),
                            owning_remote: Some(*remote_name),
                            task_id: Some(*source_index as u128),
                            ..Default::default()
                        }),
                        None,
                        None,
                    ));
                }

                let mut priority = 4.0;

                if harvester_count < hauler_count {
                    priority *= 5.0;
                }

                if current_creeps_on_source == 0 {
                    priority = 400000.0;
                }

                priority += parts_needed_on_source as f64;

                return Some(cache.spawning.create_room_spawn_request(
                    Role::RemoteHarvester,
                    body,
                    priority,
                    cost,
                    room.name(),
                    Some(CreepMemory {
                        owning_room: room.name(),
                        owning_remote: Some(*remote_name),
                        task_id: Some(*source_index as u128),
                        ..Default::default()
                    }),
                    None,
                    None,
                ));
            }
        } else if !owning_cache.remotes_with_harvester.contains(remote_name) {
            if let Some(remote_memory) = memory.remote_rooms.get_mut(remote_name) {
                if let Some(first_source) = remote_memory.sources.first() {
                    let (finished, body) = creep_sizing::miner_body(room, owning_cache, 3, false);

                    let priority = 50.0;
                    let cost = get_body_cost(&body);

                    return Some(cache.spawning.create_room_spawn_request(
                        Role::RemoteHarvester,
                        body,
                        priority,
                        cost,
                        room.name(),
                        Some(CreepMemory {
                            owning_room: room.name(),
                            owning_remote: Some(remote_name.clone()),
                            task_id: Some(0),
                            ..Default::default()
                        }),
                        None,
                        None,
                    ));
                }
            }
        }
    }
    None
}
