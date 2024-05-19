use std::cmp;

use screeps::{look, BodyPart, ErrorCode, HasPosition, Part, Room, StructureSpawn};

use crate::{memory::{RoomMemory, ScreepsMemory}, traits::room::RoomExtensions};

pub fn formulate_miner(room: &Room, memory: &mut ScreepsMemory, spawn: StructureSpawn) -> Result<(), screeps::ErrorCode> {
    let mut cost = 0;
    let mut parts = Vec::new();

    let mut room_memory = memory.get_room_mut(&room.name());

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

    let name = format!("sm-{}-{}", room_memory.creeps.len() + 1, room.name());

    if cost < room.energy_available() {
        let x = spawn.pos().x().u8();
        let y = spawn.pos().y().u8();

        let area_to_move = room.look_for_at_area(look::CREEPS, y - 1, x - 1, y + 1, x + 1);
        let spawn_result = spawn.spawn_creep(&parts, &name);

        memory.create_creep(&room.name_str(), &name, crate::memory::Role::Miner);

        return spawn_result;
    }

    Err(ErrorCode::NotEnough)
}