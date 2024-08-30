use log::info;
use screeps::{game, Part, ResourceType, Room};

use crate::{
    constants::{part_costs, PartsCost},
    memory::Role,
    room::cache::CachedRoom,
    utils::{self, get_body_cost, under_storage_gate},
};

/// Returns the parts needed for a miner creep
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn miner_body(room: &Room, cache: &CachedRoom, source_parts_needed: u8, force_max: bool, _has_container: bool) -> (bool, Vec<Part>) {
    //let mut parts = if has_container && cache.rcl <= 4 {
    //    vec![Part::Work, Part::Move]
    //} else {
    //    vec![Part::Work, Part::Carry, Part::Move]
    //};
    let mut parts = vec![Part::Work, Part::Move, Part::Carry];

    if source_parts_needed == 0 {
        info!("No parts needed for miner");
        return (true, parts); // No parts needed at all, return empty
    }

    let cost_of_stamp = 150;
    let miner_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Harvester)
        .unwrap_or(&Vec::new())
        .len();
    let base_hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::BaseHauler)
        .unwrap_or(&Vec::new())
        .len();

    let mut current_work_count = 1;
    let mut current_cost = utils::get_body_cost(&parts);

    let mut energy_to_use = if miner_count < 2 || base_hauler_count == 0 {
        room.energy_available()
    } else {
        room.energy_capacity_available()
    };

    if force_max {
        energy_to_use = room.energy_capacity_available();
    }

    while current_cost < energy_to_use {
        if current_cost + cost_of_stamp > energy_to_use || current_work_count >= source_parts_needed
        {
            break;
        }

        if current_work_count % 2 == 0 {
            parts.push(Part::Work);
            parts.push(Part::Move);

            current_cost += 150;
            current_work_count += 1;
        } else {
            parts.push(Part::Work);
            current_cost += 100;
            current_work_count += 1;
        }
    }

    // Returns if we have enough parts to fill the source, and the parts needed.
    (current_work_count >= source_parts_needed, parts)
}

pub fn mineral_miner_body(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut body = Vec::new();
    let stamp = vec![Part::Work, Part::Work, Part::Work, Part::Work, Part::Move];
    let cost = get_body_cost(&stamp);

    let max_cost = room.energy_capacity_available();
    let mut current_cost = cost;

    while current_cost < max_cost {
        if current_cost + cost > max_cost {
            break;
        }

        body.extend_from_slice(&stamp);
        current_cost += cost;
    }

    stamp
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn hauler_body(room: &Room, cache: &CachedRoom, scan_check: bool) -> Vec<Part> {
    let mut body = Vec::new();

    let hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    // Every hundo = 1C 1M
    let energy_for_haulers = match room.controller().unwrap().level() {
        1 => 100,
        2 => 300,
        3 => 400,
        4 => 500,
        5 => 800,
        6 => 800,
        // We get more spawns, so they suck up less spawn time
        7 => 1500,
        // 3 spawns, go ham.
        8 => 3000,
        _ => 100,
    };

    let tile_usage = 100;
    let mut current_energy_usage = 0;

    let (max, mut energy_to_use) = if cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len()
        > 3
        && !cache
            .creeps
            .creeps_of_role
            .get(&Role::BaseHauler)
            .unwrap_or(&Vec::new())
            .is_empty()
    {
        (false, room.energy_capacity_available())
    } else {
        (true, room.energy_available())
    };

    if scan_check && hauler_count >= 3 {
        energy_to_use = room.energy_capacity_available();
    }

    let mut energy_to_use = energy_to_use.clamp(0, energy_for_haulers);

    if cache.structures.storage.is_some()
        && max
        && cache
            .structures
            .storage
            .as_ref()
            .unwrap()
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            < 5000
    {
        // TODO: Idk, think of something better.
        energy_to_use /= 2;
    }

    while current_energy_usage < energy_to_use {
        if current_energy_usage + tile_usage > energy_to_use || body.len() >= 50 {
            break;
        }

        body.push(Part::Move);
        body.push(Part::Carry);
        current_energy_usage += tile_usage;
    }

    body
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn base_hauler_body(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let hauler_count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&Vec::new())
        .len();

    let mut storage_blocked = false;

    if let Some(storage) = &cache.structures.storage {
        if storage.store().get_used_capacity(Some(ResourceType::Energy)) < 5000 {
            storage_blocked = true;
        }
    }

    let mut max_energy = if hauler_count >= 5 && !storage_blocked {
        room.energy_capacity_available()
    } else {
        room.energy_available()
    };

    if cache.creeps.creeps_of_role(Role::Harvester) < 2 || cache.creeps.creeps_of_role(Role::Hauler) < 2 || cache.rcl < cache.max_rcl || under_storage_gate(cache, 0.8) {
        max_energy = room.energy_available();
    }

    let mut body = vec![Part::Move, Part::Carry];
    let mut cost = 100;

    let stamp_cost = if cache.rcl >= 4 { 150 } else { 100 };

    while cost < max_energy {
        if cost + stamp_cost > max_energy || body.len() >= 50 {
            break;
        }

        // Odds are, we have roads at this point.
        // So, we can expand the size.
        if cache.rcl >= 4 {
            body.push(Part::Carry);
            body.push(Part::Carry);
            body.push(Part::Move);
            cost += stamp_cost;
        } else {
            body.push(Part::Move);
            body.push(Part::Carry);
            cost += stamp_cost;
        }
    }

    body
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn storage_sitter_body(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    vec![Part::Carry, Part::Carry, Part::Carry, Part::Carry, Part::Carry, Part::Move]
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn builder_body(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let current_builders = cache
        .creeps
        .creeps_of_role
        .get(&Role::Builder)
        .unwrap_or(&Vec::new())
        .len();

    let stamp_cost = part_costs()[PartsCost::Work]
        + part_costs()[PartsCost::Move]
        + part_costs()[PartsCost::Carry];
    let max_capable = if current_builders >= 1 {
        room.energy_capacity_available()
    } else {
        room.energy_available()
    };

    let mut current_cost = part_costs()[PartsCost::Move] * 2;
    parts.push(Part::Move);
    parts.push(Part::Move);

    while current_cost < max_capable {
        if current_cost + stamp_cost > max_capable || parts.len() >= 50 {
            break;
        }

        parts.push(Part::Work);
        parts.push(Part::Move);
        parts.push(Part::Carry);
        current_cost += stamp_cost;
    }

    parts
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn repairer_body(room: &Room, parts_needed: u8, cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let current_repairers = cache
        .creeps
        .creeps_of_role
        .get(&Role::Repairer)
        .unwrap_or(&Vec::new())
        .len();

    let stamp_cost = part_costs()[PartsCost::Work]
        + part_costs()[PartsCost::Move]
        + part_costs()[PartsCost::Carry];
    let max_capable = if current_repairers >= 1 {
        room.energy_capacity_available()
    } else {
        room.energy_available()
    };

    let mut current_cost = part_costs()[PartsCost::Move] * 2;
    let mut work_count = 0;
    parts.push(Part::Move);
    parts.push(Part::Move);

    while current_cost < max_capable {
        if current_cost + stamp_cost > max_capable || work_count >= parts_needed || parts.len() >= 50 {
            break;
        }

        parts.push(Part::Work);
        parts.push(Part::Move);
        parts.push(Part::Carry);
        current_cost += stamp_cost;
        work_count += 1;
    }

    if work_count == 0 {
        return Vec::new();
    }

    parts
}

/// Returns the parts needed for a upgrader creep
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn upgrader_body(room: &Room, cache: &CachedRoom, mut target_work_parts: usize) -> Vec<Part> {
    let mut parts = Vec::new();
    let level = cache
    .structures
    .controller
    .as_ref()
    .unwrap()
    .level();

    let current_upgraders = cache.creeps.creeps_of_role(Role::Upgrader);

    if !under_storage_gate(cache, 3.0) && current_upgraders >= 1 {
        target_work_parts *= 3;
    }

    let current_work_parts = cache
        .creeps
        .creeps_of_role
        .get(&Role::Upgrader)
        .unwrap_or(&Vec::new())
        .iter()
        .map(|creep| {
            let creep = game::creeps().get(creep.as_str().to_owned()).unwrap();
            let parts = creep
                .body()
                .iter()
                .map(|part| part.part())
                .collect::<Vec<Part>>();

            parts.iter().filter(|part| **part == Part::Work).count()
        })
        .sum::<usize>();

    if current_work_parts >= target_work_parts || current_work_parts >= 65 {
        return parts;
    }

    let mut parts_needed_to_fill = target_work_parts - current_work_parts;
    if level >= 8 {
        parts_needed_to_fill = parts_needed_to_fill.clamp(0, 15);
    }


    parts.push(Part::Carry);
    parts.push(Part::Move);
    parts.push(Part::Move);
    let mut current_cost = get_body_cost(&parts);
    let mut cost_capable = room.energy_available();
    let mut max_cost = room.energy_capacity_available();

    let mut current_work_count = 0;

    let no_link_cost = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];
    let link_cost = part_costs()[PartsCost::Work];

    if !utils::under_storage_gate(cache, 2.5) && current_upgraders >= 1 {
        max_cost = room.energy_capacity_available();
        cost_capable = room.energy_capacity_available();

        parts_needed_to_fill = 50;
    }

    if current_upgraders <= 1 {
        max_cost = room.energy_available();
        cost_capable = room.energy_available();
    }

    // If we are level 5, we have a link, so we can go ham.
    let mut tick = 0;
    if level >= 5 {
        while current_cost < max_cost {
            if current_cost + link_cost >= max_cost || current_work_count >= parts_needed_to_fill || parts.len() >= 50 {
                break;
            }

            if tick % 2 == 0 {
                parts.push(Part::Work);
                parts.push(Part::Move);
                current_cost += part_costs()[PartsCost::Move] + part_costs()[PartsCost::Work];
            } else {
                parts.push(Part::Work);
                current_cost += link_cost;
            }

            tick += 1;
            current_work_count += 1;
        }
    } else {
        while current_cost < cost_capable {
            if current_cost + no_link_cost >= cost_capable || current_work_count >= parts_needed_to_fill || parts.len() >= 50 {
                break;
            }

            parts.push(Part::Work);
            parts.push(Part::Move);
            current_work_count += 1;
            current_cost += no_link_cost;
        }
    }

    if parts.contains(&Part::Work) {
        parts
    } else {
        vec![Part::Work, Part::Move, Part::Carry]
    }
}
