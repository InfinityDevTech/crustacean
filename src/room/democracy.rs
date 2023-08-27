use std::{collections::HashMap, str::FromStr};

use log::info;
use screeps::{
    find, game,
    look::{self, LookResult},
    HasPosition, HasTypedId, ObjectId, Part, Room,
};

use crate::{
    memory::{Mine, ScreepsMemory, Task},
    room::population,
};

use super::industries;

const UPGRADER_COUNT: u8 = 4;

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    // Horray, i did it better.
    let creeps = get_room_creeps_and_clean(memory, &room);
    let roommem_readonly = memory.rooms.get(&room.name().to_string()).unwrap();

    if !roommem_readonly.init {
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
            memory.rooms.get_mut(&room.name().to_string()).unwrap().avs = mining_spots.len() as u8;
            memory.rooms.get_mut(&room.name().to_string()).unwrap().mine = mining_spots;
            memory.rooms.get_mut(&room.name().to_string()).unwrap().init = true;
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
}

pub fn get_room_creeps_and_clean(memory: &mut ScreepsMemory, room: &Room) -> Vec<String> {
    let mut creeps = Vec::new();
    let mut removed_creeps = 0;
    for creep_name in &memory
        .clone()
        .rooms
        .get_mut(&room.name().to_string())
        .unwrap()
        .cs
    {
        if let Some(creep) = game::creeps().get(creep_name.to_string()) {
            if creep.spawning() {
                continue;
            }

            creeps.push(creep_name.to_string());
        } else if game::creeps().get(creep_name.to_string()).is_none() {
            removed_creeps += 1;
            let t = &memory.creeps.get(&creep_name.to_string()).unwrap().t;
            match t.clone().unwrap() {
                Task::Miner(_) => {
                    population::miner_died(memory, creep_name, &room.name().to_string())
                }
                Task::Hauler(_) => {
                    population::hauler_died(memory, creep_name, &room.name().to_string())
                }
                Task::Upgrader(_) => {
                    population::upgrader_died(memory, creep_name, &room.name().to_string())
                }
                _ => {}
            }
        }
    }
    if memory.stats.is_some() {
        memory.stats.as_mut().unwrap().crm += removed_creeps;
    }
    creeps
}

pub fn do_spawning(memory: &mut ScreepsMemory, room: &Room) {
    let binding = memory.clone();
    let roommem_readonly = binding.rooms.get(&room.name().to_string()).unwrap();
    let binding = room.find(find::MY_SPAWNS, None);
    let spawn = binding.first().unwrap();

    if population::create_miner(memory, room.clone()) {
    } else if memory.rooms.get(&room.name().to_string()).unwrap().c_c.hauler < (memory.rooms.get(&room.name().to_string()).unwrap().c_c.miner / 2) {
        let name = format!("h-{}", roommem_readonly.c_m);
        let body = [Part::Move, Part::Move, Part::Carry, Part::Work];
        let spawn_res = spawn.spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                &room.name().to_string(),
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
            memory
                .rooms
                .get_mut(&room.name().to_string())
                .unwrap()
                .c_c
                .hauler += 1;
            memory.rooms.get_mut(&room.name().to_string()).unwrap().c_m += 1;
        }
    } else if memory.rooms.get(&room.name().to_string()).unwrap().c_c.upgrader < UPGRADER_COUNT {
        let name = format!("u-{}", roommem_readonly.c_m);
        let body = [Part::Move, Part::Carry, Part::Carry, Part::Work];
        let spawn_res = spawn.spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                &room.name().to_string(),
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Upgrader(
                    ObjectId::from_str(&room.controller().unwrap().id().to_string()).unwrap(),
                )),
            );
            memory
                .rooms
                .get_mut(&room.name().to_string())
                .unwrap()
                .c_c
                .upgrader += 1;
            memory.rooms.get_mut(&room.name().to_string()).unwrap().c_m += 1;
        }
    }
}
