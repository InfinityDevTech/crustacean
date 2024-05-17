use std::cmp;

use screeps::{BodyPart, Part, Room};

pub fn formulate_miner(room: &Room) -> (u32, Vec<Part>) {
    let mut cost = 0;
    let mut parts = Vec::new();

    parts.push(Part::Move);
    parts.push(Part::Move);
    cost += 100;
    parts.push(Part::Carry);
    cost += 50;

    let energy_capacity = room.energy_capacity_available() - cost;
    let max_work_parts = energy_capacity / 100;

    for _ in 0.. cmp::min(max_work_parts, 10) {
        parts.push(Part::Work);
        cost += 100;
    }

    (cost, parts)
}