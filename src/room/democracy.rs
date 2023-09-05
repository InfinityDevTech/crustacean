use std::{collections::HashMap, str::FromStr, cmp};

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

use super::{creeps, tower};

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
    memory.stats.get_room(&room.name_str()).energy_available = room.energy_available() as u64;
    memory.stats.get_room(&room.name_str()).energy_capacity_available = room.energy_capacity_available() as u64;

    if game::time() % 1000 == 0 {
        game::notify(&format!("Room name: {} - RCL progress: {:.2}", room.name_str(), room.controller().unwrap().progress() as f64 / room.controller().unwrap().progress_total() as f64 * 100.0), None)
    }
    // Horray, i did it better.
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
            roommem.available_mining = mining_spots.len() as u8;
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

    creeps::market::run_creeps(&room, memory);
    tower::run_towers(&room);

    do_spawning(memory, &room);
    memory.stats.cpu.rooms += game::cpu::get_used() - starting_cpu;
    memory.stats.rooms.get_mut(&room.name_str()).unwrap().cpu += game::cpu::get_used() - starting_cpu;
}

pub fn do_spawning(memory: &mut ScreepsMemory, room: &Room) {
    let binding = memory.clone();
    let roommem_readonly = binding.rooms.get(&room.name_str()).unwrap();
    let binding = room.find(find::MY_SPAWNS, None);
    let spawn = binding.first().unwrap();
    let room_name = &room.name_str();

    if population::create_miner(memory, room.clone()) {
    } else if memory.get_room(&room.name_str()).get_creeps_by_role("hauler").len() < memory.get_room(&room.name_str()).get_creeps_by_role("miner").len() {
        let name = format!("h-{}", roommem_readonly.creeps_made);
        let body = get_max_body(room.clone(), &[Part::Move, Part::Carry], 5);
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
            memory.get_room(&room.name_str()).creeps.insert(name.to_string(), "hauler".to_string());
            memory.get_room(&room.name_str()).creeps_made += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
        }
    } else if (memory.get_room(&room.name_str()).get_creeps_by_role("upgrader").len() as u8) < UPGRADER_COUNT {
        let name = format!("u-{}", roommem_readonly.creeps_made);
        let body = get_max_body(room.clone(), &[Part::Move, Part::Move, Part::Carry, Part::Carry, Part::Work], 9);
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
            memory.get_room(&room.name().to_string()).creeps.insert(name.to_string(), "upgrader".to_string());
            memory.get_room(&room.name_str()).creeps_made += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
        }
    } else if (memory.get_room(&room.name_str()).get_creeps_by_role("builder").len() as u8) < BUILDER_COUNT {
        let name = format!("b-{}", roommem_readonly.creeps_made);
        let body = get_max_body(room.clone(), &[Part::Move, Part::Move, Part::Carry, Part::Carry, Part::Work], 9);
        let spawn_res = spawn.spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                room_name,
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Builder()),
            );
            memory.get_room(&room.name_str()).creeps.insert(name.to_string(), "builder".to_string());
            memory.get_room(&room.name_str()).creeps_made += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
        }
    } else if (memory.get_room(&room.name_str()).get_creeps_by_role("attacker").len() as u8) < 3 {
        let name = format!("a-{}", roommem_readonly.creeps_made);
        let body = get_max_body(room.clone(), &[Part::Attack, Part::Tough, Part::Move, Part::Move], 9);
        let spawn_res = spawn.spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                room_name,
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Attacker()),
            );
            memory.get_room(&room.name_str()).creeps.insert(name.to_string(), "attacker".to_string());
            memory.get_room(&room.name_str()).creeps_made += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
        }
    } else if (memory.get_room(&room.name_str()).get_creeps_by_role("healer").len() as u8) < 3 {
        let name = format!("h-{}", roommem_readonly.creeps_made);
        let body = get_max_body(room.clone(), &[Part::Move, Part::Heal], 50);
        let spawn_res = spawn.spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                room_name,
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Healer()),
            );
            memory.get_room(&room.name_str()).creeps.insert(name.to_string(), "healer".to_string());
            memory.get_room(&room.name_str()).creeps_made += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
        }
    }
}

pub fn get_max_body(room: Room, body: &[Part], max: u8) -> Vec<Part> {
    let mut max_body = Vec::new();
    let body = body.to_vec();
    let body_cost = body.iter().map(|x| x.cost()).sum::<u32>();
    let max_cycles = cmp::min(((room.energy_available() / body_cost) as f32).ceil() as u8, max);
    for _ in 0..max_cycles as u32 {
        max_body.append(&mut body.clone());
    }
    max_body
}
