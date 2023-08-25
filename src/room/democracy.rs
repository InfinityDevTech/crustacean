use std::str::FromStr;

use log::info;
use screeps::{find, game, HasPosition, HasTypedId, ObjectId, Part, Room};

use crate::memory::{ScreepsMemory, Task};

use super::industries;

const MINER_COUNT: u8 = 2;
const HAULER_COUNT: u8 = 2;

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    let mut creeps = Vec::new();
    let mut to_remove = Vec::new();

    // Horray, i did it better.
    for name in memory
        .rooms
        .get_mut(&room.name().to_string())
        .unwrap()
        .creeps
        .clone()
        .keys()
    {
        for creep_name in memory.clone().rooms.get_mut(&room.name().to_string()).unwrap().creeps.keys() {
            info!("Name, {}", creep_name);
            if let Some(creep) = game::creeps().get(creep_name.to_string()) {
                info!("Found Crep");
                if creep.spawning() {continue;}

                creeps.push(name.to_string());
            } else if game::creeps().get(creep_name.to_string()).is_none() {
                let t = &memory.clone().rooms.get_mut(&room.name().to_string()).unwrap().creeps.get_mut(creep_name).unwrap().work.clone().unwrap().task.unwrap();
                    match t {
                        Task::Miner(_) => {
                            info!("Subtract 1");
                            memory.rooms.get_mut(&room.name().to_string()).unwrap().creep_count.miner -= 1;
                        },
                        Task::Hauler(_) => {
                            info!("Subtract 1 -2 ");
                            memory.rooms.get_mut(&room.name().to_string()).unwrap().creep_count.hauler -= 1;
                        },
                        _ => {},
                    }
                to_remove.push(creep_name.to_string());
            }
        }
    }
    memory
        .rooms
        .get_mut(&room.name().to_string())
        .unwrap()
        .creeps
        .retain(|x, _| !to_remove.contains(x));

    industries::mining::pre_market(&room, creeps, memory);

    info!("Post market");

    if memory
        .rooms
        .get_mut(&room.name().to_string())
        .unwrap()
        .creep_count
        .miner
        < MINER_COUNT
    {
        let spawners = room.find(find::MY_SPAWNS, None);
        let n = &uuid::Uuid::new_v4().to_string();
        let spawn = spawners
        .first()
        .unwrap()
        .spawn_creep(&[Part::Move, Part::Work, Part::Work], n);
        if let Ok(res) = spawn
        {
            info!("Ron");
            memory
                .rooms
                .get_mut(&room.name().to_string())
                .unwrap()
                .creep_count
                .miner += 1;
            memory.create_creep(
                &room.name().to_string(),
                n,
                Task::Miner(
                    spawners
                        .first()
                        .unwrap()
                        .pos()
                        .find_closest_by_range(find::SOURCES)
                        .unwrap()
                        .id()
                ),
            );
        } else if let Err(e) = spawn {
            info!("Error spawning creep: {:?}", e);
        }
    } else if memory
        .rooms
        .get_mut(&room.name().to_string())
        .unwrap()
        .creep_count
        .hauler
        < HAULER_COUNT
    {
        let spawners = room.find(find::MY_SPAWNS, None);
        let n = &uuid::Uuid::new_v4().to_string();
        let spawn = spawners
        .first()
        .unwrap()
        .spawn_creep(&[Part::Move, Part::Move, Part::Carry, Part::Work], n);
        if let Ok(res) = spawn {
            info!("Sub 1");
            memory
                .rooms
                .get_mut(&room.name().to_string())
                .unwrap()
                .creep_count
                .hauler += 1;
            memory.create_creep(
                &room.name().to_string(),
                n,
                Task::Hauler(
                    ObjectId::from_str(&spawners.first().unwrap().id().to_string()).unwrap(),
                ),
            )
        } else if let Err(e) = spawn {
            info!("Error spawning creep: {:?}", e);
        }
    }
}
