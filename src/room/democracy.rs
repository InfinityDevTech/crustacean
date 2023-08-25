use std::str::FromStr;

use log::info;
use screeps::{find, game, HasPosition, HasTypedId, ObjectId, Part, Room};

use crate::{memory::{ScreepsMemory, Task}, movement::creep};

use super::industries;

const MINER_COUNT: u8 = 2;
const HAULER_COUNT: u8 = 2;
const UPGRADER_COUNT: u8 = 2;

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    // Horray, i did it better.
    let creeps = get_creeps_and_clean(memory, &room);

    industries::mining::pre_market(&room, creeps.clone(), memory);
    industries::construction::pre_market(&room, creeps.clone(), memory);

    do_spawning(memory, &room);
}

pub fn get_creeps_and_clean(memory: &mut ScreepsMemory, room: &Room) -> Vec<String> {
    let mut to_remove = Vec::new();
    let mut creeps = Vec::new();
    for creep_name in memory
        .rooms
        .get_mut(&room.name().to_string())
        .unwrap()
        .creeps
        .clone()
        .keys()
    {
        info!("Name, {}", creep_name);
        if let Some(creep) = game::creeps().get(creep_name.to_string()) {
            info!("Found Crep");
            if creep.spawning() {
                continue;
            }

            creeps.push(creep_name.to_string());
        } else if game::creeps().get(creep_name.to_string()).is_none() {
            let t = &memory
                .clone()
                .rooms
                .get_mut(&room.name().to_string())
                .unwrap()
                .creeps
                .get_mut(creep_name)
                .unwrap()
                .work
                .clone()
                .unwrap()
                .task
                .unwrap();
            match t {
                Task::Miner(_) => {
                    info!("Subtract 1");
                    memory
                        .rooms
                        .get_mut(&room.name().to_string())
                        .unwrap()
                        .creep_count
                        .miner -= 1;
                }
                Task::Hauler(_) => {
                    info!("Subtract 1 -2 ");
                    memory
                        .rooms
                        .get_mut(&room.name().to_string())
                        .unwrap()
                        .creep_count
                        .hauler -= 1;
                }
                _ => {}
            }
            to_remove.push(creep_name.to_string());
        }
    }
    memory
        .rooms
        .get_mut(&room.name().to_string())
        .unwrap()
        .creeps
        .retain(|x, _| !to_remove.contains(x));
    creeps
}

pub fn do_spawning(memory: &mut ScreepsMemory, room: &Room) {
    let creep_count = &mut memory
        .rooms
        .get_mut(&room.name().to_string())
        .unwrap()
        .creep_count;

    let spawner = room.find(find::MY_SPAWNS, None);
    let spawner = spawner.first().unwrap();
    info!("test {:?}", creep_count);
    if creep_count.miner < MINER_COUNT {
        let n = &uuid::Uuid::new_v4().to_string();
        let spawn = spawner.spawn_creep(&[Part::Move, Part::Work, Part::Work], n);
        if spawn.is_ok() {
            creep_count.miner += 1;
            memory.create_creep(
                &room.name().to_string(),
                n,
                Task::Miner(
                    spawner
                        .pos()
                        .find_closest_by_range(find::SOURCES)
                        .unwrap()
                        .id(),
                ),
            );
        }
    } else if creep_count.hauler < HAULER_COUNT {
        let n = &uuid::Uuid::new_v4().to_string();
        let spawn = spawner.spawn_creep(&[Part::Move, Part::Move, Part::Carry, Part::Work], n);
        if spawn.is_ok() {
            creep_count.hauler += 1;
            memory.create_creep(
                &room.name().to_string(),
                n,
                Task::Hauler(ObjectId::from_str(&spawner.id().to_string()).unwrap()),
            )
        }
    } else if creep_count.upgrader < UPGRADER_COUNT {
        let n = &uuid::Uuid::new_v4().to_string();
        let spawn = spawner.spawn_creep(&[Part::Move, Part::Move, Part::Carry, Part::Work], n);
        if spawn.is_ok() {
            creep_count.upgrader += 1;
            memory.create_creep(
                &room.name().to_string(),
                n,
                Task::Upgrader(ObjectId::from_str(&room.controller().unwrap().id().to_string()).unwrap()),
            )
        }
    }
}
