use log::info;
use screeps::{game, Part, Room, SharedCreepProperties};

use crate::{constants::{part_costs, PartsCost}, memory::Role, room::cache::tick_cache::CachedRoom};


/// Returns the parts needed for a miner creep
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn miner(room: &Room, cache: &CachedRoom, source_parts_needed: u8) -> Vec<Part> {
    let mut parts = Vec::new();

    if source_parts_needed == 0 {
        info!("No parts needed for miner");
        return parts; // No parts needed at all, return empty
    }

    let cost_of_stamp = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move];
    let energy_stored = room.energy_available();
    let max_energy = room.energy_capacity_available();

    let miner_count = cache.creeps.creeps_of_role.get(&Role::Miner).unwrap_or(&Vec::new()).len();

    if miner_count > 2 {
        let mut current_cost = part_costs()[PartsCost::Carry] + cost_of_stamp;
        let mut work_part_count = 0;
        parts.push(Part::Carry);
        parts.push(Part::Work);
        parts.push(Part::Move);

        while current_cost < max_energy {
            if work_part_count % 2 == 0 {
                if current_cost + cost_of_stamp > max_energy || work_part_count >= source_parts_needed {
                    break;
                }

                parts.push(Part::Move);
                parts.push(Part::Work);
                current_cost += part_costs()[PartsCost::Move] + part_costs()[PartsCost::Work];

                work_part_count += 1;
            } else {

                let cost = part_costs()[PartsCost::Work];
                if current_cost + cost > max_energy || work_part_count >= source_parts_needed {
                    break;
                }

                parts.push(Part::Work);
                current_cost += part_costs()[PartsCost::Work];

                work_part_count += 1;
            }

        }

        return parts;
    } else {
        let mut current_cost = part_costs()[PartsCost::Carry] + cost_of_stamp;
        let mut work_part_count = 0;
        parts.push(Part::Carry);
        parts.push(Part::Work);
        parts.push(Part::Move);

        while current_cost < energy_stored {
            if current_cost + cost_of_stamp > energy_stored || work_part_count >= source_parts_needed {
                break;
            }

            if work_part_count % 2 == 0 {
                parts.push(Part::Move);
                parts.push(Part::Work);
                current_cost += part_costs()[PartsCost::Move] + part_costs()[PartsCost::Work];

                work_part_count += 1;
            } else {

                let cost = part_costs()[PartsCost::Work];
                if current_cost + cost > max_energy || work_part_count >= source_parts_needed {
                    break;
                }

                parts.push(Part::Work);
                current_cost += part_costs()[PartsCost::Work];

                work_part_count += 1;
            }
        }
    }

    parts
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn hauler(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut body = Vec::new();

    let currently_capable = room.energy_available();
    let max_capable = room.energy_capacity_available();

    let hauler_count = cache.creeps.creeps_of_role.get(&Role::Hauler).unwrap_or(&Vec::new()).len();

    let stamp_cost = part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];

    let max_parts = 25;

    if hauler_count > 3 {
        let mut current_cost = stamp_cost;
        body.push(Part::Move);
        body.push(Part::Move);

        while current_cost < max_capable {
            if current_cost + part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry] > max_capable {
                break;
            }

            body.push(Part::Move);
            body.push(Part::Carry);
            current_cost += stamp_cost;

            if body.len() >= max_parts {
                break;
            }
        }
    } else {
        let mut current_cost = stamp_cost;
        body.push(Part::Move);
        body.push(Part::Move);

        while current_cost < currently_capable {
            if current_cost + stamp_cost > currently_capable {
                break;
            }

            body.push(Part::Move);
            body.push(Part::Carry);
            current_cost += stamp_cost;

            if body.len() >= max_parts {
                break;
            }
        }
    }

    body
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn builder(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let stamp_cost = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];
    let max_capable = room.energy_capacity_available();

    let mut current_cost = part_costs()[PartsCost::Move] * 2;
    let mut work_part_count = 0;
    parts.push(Part::Move);
    parts.push(Part::Move);

    while current_cost < max_capable {
        if current_cost + stamp_cost > max_capable {
            break;
        }

        parts.push(Part::Work);
        parts.push(Part::Move);
        parts.push(Part::Carry);
        work_part_count += 1;
        current_cost += stamp_cost;
    }

    parts
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn repairer(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let stamp_cost = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];
    let max_capable = room.energy_capacity_available();

    let mut current_cost = part_costs()[PartsCost::Move] * 2;
    let mut part_count = 25;
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

        if parts.len() >= part_count {
            break;
        }
    }

    parts
}

/// Returns the parts needed for a upgrader creep
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn upgrader(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let room_current_rcl = cache.structures.controller.as_ref().unwrap().controller.level();
    let target_work_parts = match room_current_rcl {
        1 => 1,
        2 => 5,
        3 => 12,
        4 => 15,
        5 => 20,
        6 => 25,
        7 => 25,
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