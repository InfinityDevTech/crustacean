use log::info;
use screeps::{game, Room};

use crate::memory::ScreepsMemory;

pub fn pre_market(
    room: &Room,
    creeps: Vec<String>,
    memory: &mut ScreepsMemory,
) {

    info!("[INDUSTRIES] Running mining industry...");

    for name in creeps {
        let creepmem = memory.creeps.get_mut(&name).unwrap();
        if let Some(task) = &creepmem.t {
                let creep = game::creeps().get(name).unwrap();
                match task {
                    crate::memory::Task::Miner(source_id) => {
                        if let Some(source) = source_id.resolve() {
                            crate::roles::local::harvester::harvest(&creep, creepmem, source)
                        } else {
                            creepmem.t = None;
                        }
                    }
                    crate::memory::Task::Hauler(building_id) => {
                        if let Some(building) = building_id.resolve() {
                            crate::roles::local::hauler::haul(&creep, creepmem, building)
                        }
                    }
                    _ => {},
                }
        }
    }
}
