use log::info;
use screeps::{Room, game, SharedCreepProperties};

use crate::{memory::ScreepsMemory, traits::room::RoomExtensions, room::{population, creeps::enemy}, cache::ScreepsCache};

use super::local;

pub fn run_creeps(room: &Room, memory: &mut ScreepsMemory, cache: &mut ScreepsCache) {
    info!("[CREEPS] Running creeps in room: {}", room.name_str());
    let creeps = memory.get_room(&room.name_str()).creeps.clone();
    for (creep_name, _job) in creeps {
        let starting_cpu = game::cpu::get_used();
        if let Some(creep) = game::creeps().get(creep_name.clone()) {
            if creep.spawning() {
                continue;
            }
        info!("Running creep {}", creep_name);

        let creep_memory = memory.get_creep(&creep_name);

        if let Some(task) = &creep_memory.t {
            match task {
                crate::memory::Task::Miner(source_id) => {
                    if let Some(source) = source_id.resolve() {
                        local::harvester::run_creep(&creep, memory, source, cache)
                    }
                },
                crate::memory::Task::Hauler(structure_id) => {
                    if let Some(structure) = structure_id.resolve() {
                        local::hauler::run_creep(&creep, creep_memory, structure, cache)
                    }
                },
                crate::memory::Task::Upgrader(structure_id) => {
                    if let Some(structure) = structure_id.resolve() {
                        local::upgrader::run_creep(&creep, creep_memory, structure, cache)
                    }
                },
                crate::memory::Task::Attacker() => enemy::attacker::run_creep(&creep, creep_memory),
                crate::memory::Task::Healer() => enemy::healer::run_creep(&creep, creep_memory, cache),
                crate::memory::Task::Builder() => local::builder::run_creep(&creep, creep_memory, cache),
                _ => {}
            }
            info!("Used CPU: {}", game::cpu::get_used() - starting_cpu);
        }
    } else {
        let creep_mem = memory.get_creep(&creep_name);
            if let crate::memory::Task::Miner(_source) = creep_mem.t.clone().expect("Failed to get creep task from memory") {
                    population::miner_died(memory, &creep_name, &room.name_str());
            }
            memory.creeps.remove(&creep_name);
            memory.get_room(&room.name_str()).creeps.remove(&creep_name);
            continue;
    }
    }
}