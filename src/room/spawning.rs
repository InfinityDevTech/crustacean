use log::warn;
use screeps::{game, Part};

use crate::memory::ScreepsMemory;

pub fn run_spawns(memory: &mut ScreepsMemory) {
    let mut additional = 0;
    for spawn in game::spawns().values() {
        // Default body for now, will be sorted out later.
        let body = [Part::Move, Part::Move, Part::Carry, Part::Work];
        if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
            let name = format!("{}-{}", game::time(), additional);
            match spawn.spawn_creep(&body, &name) {
                Ok(()) => {
                    additional += 1;
                    memory.create_creep(&name, spawn.room().unwrap());
                }
                Err(e) => warn!("Couldn't spawn: {:?}", e),
            }
        }
    }
}