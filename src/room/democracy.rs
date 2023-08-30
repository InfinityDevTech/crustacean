use std::{collections::HashMap, str::FromStr};

use log::info;
use screeps::{
    find, game,
    look::{self, LookResult},
    HasPosition, HasTypedId, ObjectId, Part, Room,
};

use crate::{
    memory::{Mine, ScreepsMemory, Task},
    room::population, traits::room::RoomExtensions,
};

use super::industries;

const UPGRADER_COUNT: u8 = 8;
const BUILDER_COUNT: u8 = 4;

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    let starting_cpu = game::cpu::get_used();
    if memory.stats.rooms.get(&room.name_str()).is_none() {
        memory.stats.create_room(&room.name_str(), room.controller().unwrap().level());
    }
    memory.stats.get_room(&room.name_str()).cpu = 0.0;
    memory.stats.get_room(&room.name_str()).mining = 0.0;
    memory.stats.get_room(&room.name_str()).construction = 0.0;
    memory.stats.get_room(&room.name_str()).energy_harvested = 0;
    // Horray, i did it better.
    let creeps = get_room_creeps_and_clean(memory, &room);
    let roommem = memory.get_room(&room.name_str());

    if !roommem.init {
        if game::cpu::bucket() >= 100 {
            info!("Initialising room: {}", room.name().to_string());
            let sources = room.find(find::SOURCES, None);
            let mut mining_spots = HashMap::new();
            for spot in sources {
                let x = spot.pos().x().u8();
                let y = spot.pos().y().u8();
                let areas = room.look_for_at_area(look::TERRAIN, y - 1, x - 1, y + 1, x + 1);
                let mut available_spots = 0;
                for area in areas {
                    if let LookResult::Terrain(screeps::Terrain::Plain) = area.look_result {
                        available_spots += 1;
                    } else if let LookResult::Terrain(screeps::Terrain::Swamp) = area.look_result {
                        available_spots += 1;
                    }
                }
                mining_spots.insert(
                    spot.id(),
                    Mine {
                        u: 0,
                        s: available_spots,
                    },
                );
            }
            roommem.avs = mining_spots.len() as u8;
            roommem.mine = mining_spots;
            roommem.init = true;
        } else {
            info!(
                "CPU bucket is too low to initialise room: {}",
                room.name().to_string()
            );
            return;
        }
    }

    industries::mining::pre_market(&room, creeps.clone(), memory);
    industries::construction::pre_market(&room, creeps.clone(), memory);

    do_spawning(memory, &room);
    memory.stats.cpu.rooms += game::cpu::get_used() - starting_cpu;
    memory.stats.rooms.get_mut(&room.name_str()).unwrap().cpu += game::cpu::get_used() - starting_cpu;
}

pub fn get_room_creeps_and_clean(memory: &mut ScreepsMemory, room: &Room) -> Vec<String> {
    let mut creeps = Vec::new();
    let mut removed_creeps = 0;
    for creep_name in &memory
        .clone()
        .rooms
        .get_mut(&room.name_str())
        .unwrap()
        .cs
    {
        if let Some(creep) = game::creeps().get(creep_name.to_string()) {
            if creep.spawning() {
                continue;
            }

            creeps.push(creep_name.to_string());
        } else {
            removed_creeps += 1;
            match &memory
                .creeps
                .get(&creep_name.to_string())
                .unwrap()
                .t
                .clone()
                .unwrap()
            {
                Task::Miner(_) => {
                    population::miner_died(memory, creep_name, &room.name_str())
                }
                Task::Hauler(_) => {
                    population::hauler_died(memory, creep_name, &room.name_str())
                }
                Task::Upgrader(_) => {
                    population::upgrader_died(memory, creep_name, &room.name_str())
                }
                Task::Builder() => {
                    population::builder_died(memory, creep_name, &room.name_str())
                }
                _ => {}
            }
            memory.creeps.remove(creep_name);
        }
    }
    memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_removed += removed_creeps;
    creeps
}

pub fn do_spawning(memory: &mut ScreepsMemory, room: &Room) {
    let binding = memory.clone();
    let roommem_readonly = binding.rooms.get(&room.name_str()).unwrap();
    let binding = room.find(find::MY_SPAWNS, None);
    let spawn = binding.first().unwrap();
    let room_name = &room.name_str();

    if population::create_miner(memory, room.clone()) {
    } else if memory.get_room(&room.name_str()).c_c.hauler
        <= ((memory.get_room(&room.name_str()).c_c.miner / 2) as f32).round() as u8
    {
        let name = format!("h-{}", roommem_readonly.c_m);
        let body = [Part::Move, Part::Move, Part::Carry, Part::Work];
        let spawn_res = spawn.spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                room_name,
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Hauler(
                    ObjectId::from_str(
                        &room
                            .find(find::MY_SPAWNS, None)
                            .first()
                            .unwrap()
                            .id()
                            .to_string(),
                    )
                    .unwrap(),
                )),
            );
            memory.get_room(&room.name_str()).c_c.hauler += 1;
            memory.get_room(&room.name_str()).c_m += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
        }
    } else if memory.get_room(&room.name_str()).c_c.upgrader < UPGRADER_COUNT {
        let name = format!("u-{}", roommem_readonly.c_m);
        let body = [Part::Move, Part::Move, Part::Carry, Part::Carry, Part::Work];
        let spawn_res = spawn.spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                room_name,
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Upgrader(
                    ObjectId::from_str(&room.controller().unwrap().id().to_string()).unwrap(),
                )),
            );
            memory
                .rooms
                .get_mut(&room.name_str())
                .unwrap()
                .c_c
                .upgrader += 1;
            memory.get_room(&room.name_str()).c_m += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
        }
    } else if memory.get_room(&room.name_str()).c_c.builder < BUILDER_COUNT {
        let name = format!("b-{}", roommem_readonly.c_m);
        let body = [Part::Move, Part::Carry, Part::Carry, Part::Work];
        let spawn_res = spawn.spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                room_name,
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Builder()),
            );
            memory.get_room(&room.name_str()).c_c.builder += 1;
            memory.get_room(&room.name_str()).c_m += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
        }
    }
}
