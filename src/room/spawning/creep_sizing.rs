use screeps::{game, Part, Room};

use crate::{constants::{part_costs, PartsCost}, memory::Role, room::cache::tick_cache::CachedRoom};

/// Returns the parts needed for a miner creep
pub fn miner(room: &Room, cache: &CachedRoom, source_parts_needed: u8) -> Vec<Part> {
    let mut parts = Vec::new();

    if source_parts_needed == 0 {
        return parts; // No parts needed at all, return empty
    }

    let cost_of_stamp = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move];
    let energy_stored = room.energy_available();
    let max_energy = room.energy_capacity_available();

    let has_miner = !cache.creeps.creeps_of_role.get(&Role::Miner).unwrap_or(&Vec::new()).is_empty();

    if has_miner {
        let mut current_cost = part_costs()[PartsCost::Carry];
        let mut work_part_count = 0;
        parts.push(Part::Carry);

        while current_cost < max_energy {
            if current_cost + cost_of_stamp > max_energy || work_part_count >= source_parts_needed {
                break;
            }

            parts.push(Part::Work);
            parts.push(Part::Move);
            current_cost += cost_of_stamp;
            work_part_count += 1;
        }

        return parts;
    } else {
        let mut current_cost = part_costs()[PartsCost::Carry];
        let mut work_part_count = 0;
        parts.push(Part::Carry);

        while current_cost < energy_stored {
            if current_cost + cost_of_stamp > energy_stored || work_part_count >= source_parts_needed {
                break;
            }

            parts.push(Part::Work);
            parts.push(Part::Move);
            current_cost += cost_of_stamp;
            work_part_count += 1;
        }
    }
    parts
}

pub fn hauler(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut body = Vec::new();

    let currently_capable = room.energy_available();
    let max_capable = room.energy_capacity_available();

    let hauler_count = cache.creeps.creeps_of_role.get(&Role::Hauler).unwrap_or(&Vec::new()).len();

    let stamp_cost = part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];
    let min_cost = part_costs()[PartsCost::Move] + part_costs()[PartsCost::Carry];

    if hauler_count > 3 {
        let mut current_cost = stamp_cost;
        body.push(Part::Move);
        body.push(Part::Carry);

        while current_cost < max_capable {
            if current_cost + part_costs()[PartsCost::Move] > max_capable {
                break;
            }

            body.push(Part::Move);
            body.push(Part::Carry);
            current_cost += stamp_cost;
        }
    } else {
        let mut current_cost = stamp_cost;
        body.push(Part::Move);
        body.push(Part::Carry);

        while current_cost < currently_capable {
            if current_cost + stamp_cost > currently_capable {
                break;
            }

            body.push(Part::Move);
            body.push(Part::Carry);
            current_cost += stamp_cost;
        }
    }

    body
}

/// Returns the parts needed for a upgrader creep
pub fn upgrader(room: &Room, cache: &CachedRoom) -> Vec<Part> {
    let mut parts = Vec::new();

    let room_current_rcl = cache.structures.controller.as_ref().unwrap().controller.level();
    let target_work_parts = match room_current_rcl {
        1 => 1,
        2 => 3,
        3 => 5,
        4 => 7,
        5 => 12,
        6 => 16,
        7 => 20,
        8 => 5,
        _ => 1,
    };

    let current_work_parts = cache.creeps.creeps_of_role.get(&Role::Upgrader).unwrap_or(&Vec::new()).iter().map(|creep| {
        let creep = game::creeps().get(creep.as_str().to_owned()).unwrap();
        let parts = creep.body().iter().map(|part| part.part()).collect::<Vec<Part>>();
        parts.iter().filter(|part| **part == Part::Work).count()
    }).sum::<usize>();

    let parts_needed_to_fill = target_work_parts - current_work_parts;
    let stamp_cost = part_costs()[PartsCost::Work] + part_costs()[PartsCost::Move];
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

    parts
}