use std::cmp;

use log::info;
use screeps::{game, Part, Room};

use crate::{constants::{part_costs, PartsCost}, memory::Role, room::cache::tick_cache::CachedRoom, utils};


/// Returns the parts needed for a miner creep
//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn miner_body(room: &Room, cache: &CachedRoom, source_parts_needed: u8) -> Vec<Part> {
    let mut parts = vec![Part::Work, Part::Carry, Part::Move];

    if source_parts_needed == 0 {
        info!("No parts needed for miner");
        return parts; // No parts needed at all, return empty
    }

    let cost_of_stamp = 150;
    let miner_count = cache.creeps.creeps_of_role.get(&Role::Harvester).unwrap_or(&Vec::new()).len();

    let mut current_work_count = 1;
    let mut current_cost = utils::get_body_cost(&parts);

    let energy_to_use = if miner_count < 2 {
        room.energy_available()
    } else {
        room.energy_capacity_available()
    };

    while current_cost < energy_to_use {
        if current_cost + cost_of_stamp > energy_to_use || current_work_count >= source_parts_needed {
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

    parts
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn hauler_body(room: &Room) -> Vec<Part> {
    let mut body = Vec::new();

    // Every hundo = 1C 1M
    let energy_for_haulers = match room.controller().unwrap().level() {
        1 => 100,
        2 => 300,
        3 => 400,
        4 => 500,
        5 => 1000,
        6 => 1200,
        // We get more spawns, so they suck up less spawn time
        7 => 2000,
        // 3 spawns, go ham.
        8 => 3000,
        _ => 100,
    };

    let tile_usage = 100;
    let mut current_energy_usage = 0;

    let energy_to_use = cmp::min(room.energy_available(), energy_for_haulers);
    while current_energy_usage < energy_to_use {
        if current_energy_usage + tile_usage > energy_to_use {
            break;
        }

        body.push(Part::Move);
        body.push(Part::Carry);
        current_energy_usage += tile_usage;
    }

    body
}

pub fn base_hauler_body(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let hauler_count = cache.creeps.creeps_of_role.get(&Role::BaseHauler).unwrap_or(&Vec::new()).len();

    let max_energy = if hauler_count > 0 {
        room.energy_capacity_available()
    } else {
        room.energy_available()
    };

    info!("BH Energy {}", max_energy);

    let mut body = Vec::new();
    let mut cost = 0;

    let stamp_cost = if cache.rcl >= 4 {
        150
    } else {
        100
    };

    while cost < max_energy {
        if cost + stamp_cost > max_energy {
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
pub fn builder_body(room: &Room, _cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let stamp_cost = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];
    let max_capable = room.energy_capacity_available();

    let mut current_cost = part_costs()[PartsCost::Move] * 2;
    parts.push(Part::Move);
    parts.push(Part::Move);

    while current_cost < max_capable {
        if current_cost + stamp_cost > max_capable {
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
pub fn repairer_body(room: &Room, parts_needed: u8, _cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let stamp_cost = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];
    let max_capable = room.energy_capacity_available();

    let mut current_cost = part_costs()[PartsCost::Move] * 2;
    let mut work_count = 0;
    parts.push(Part::Move);
    parts.push(Part::Move);

    while current_cost < max_capable {
        if current_cost + stamp_cost > max_capable || work_count >= parts_needed {
            break;
        }

        parts.push(Part::Work);
        parts.push(Part::Move);
        parts.push(Part::Carry);
        current_cost += stamp_cost;
        work_count += 1;
    }

    parts
}

/// Returns the parts needed for a upgrader creep
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn upgrader_body(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let room_current_rcl = cache.structures.controller.as_ref().unwrap().controller.level();
    let target_work_parts = match room_current_rcl {
        1 => 4,
        2 => 7,
        3 => 12,
        4 => 15,
        5 => 20,
        6 => 30,
        7 => 40,
        8 => 5,
        _ => 1,
    };


    let current_work_parts = cache.creeps.creeps_of_role.get(&Role::Upgrader).unwrap_or(&Vec::new()).iter().map(|creep| {
        let creep = game::creeps().get(creep.as_str().to_owned()).unwrap();
        let parts = creep.body().iter().map(|part| part.part()).collect::<Vec<Part>>();

        parts.iter().filter(|part| **part == Part::Work).count()
    }).sum::<usize>();

    if current_work_parts >= target_work_parts {
        return parts;
    }

    let parts_needed_to_fill = target_work_parts - current_work_parts;

    let stamp_cost = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];
    let cost_capable = room.energy_available();
    let max_capable = room.energy_capacity_available();

    let has_upgrader = !cache.creeps.creeps_of_role.get(&Role::Upgrader).unwrap_or(&Vec::new()).is_empty();

    if has_upgrader {
        let mut current_cost = part_costs()[PartsCost::Carry];
        let mut work_part_count = 0;
        parts.push(Part::Carry);

        while current_cost < max_capable {
            if current_cost + stamp_cost > max_capable {
                break;
            }

            if work_part_count >= parts_needed_to_fill {
                break;
            }

            parts.push(Part::Work);
            parts.push(Part::Carry);
            parts.push(Part::Move);
            work_part_count += 1;
            current_cost += stamp_cost;
        }
    } else {
        let mut current_cost = part_costs()[PartsCost::Carry];
        parts.push(Part::Carry);

        while current_cost < cost_capable {
            if current_cost + stamp_cost > cost_capable {
                break;
            }

            parts.push(Part::Work);
            parts.push(Part::Move);
            current_cost += stamp_cost;
        }
    }

    if parts.contains(&Part::Work) {
        parts
    } else {
        vec![Part::Work, Part::Move, Part::Carry]
    }
}