use log::info;
use screeps::{game, Room};

use crate::memory::ScreepsMemory;

pub fn pre_market(
    room: &Room,
    creeps: Vec<String>,
    memory: &mut ScreepsMemory,
) {
    let roommem = memory.rooms.get_mut(&room.name().to_string()).unwrap();

    for name in creeps {
        let creepmem = roommem.creeps.get_mut(&name).unwrap();
        if let Some(work) = &creepmem.work {
            if let Some(task) = &work.task {
                let creep = game::creeps().get(name).unwrap();
                match task {
                    crate::memory::Task::Miner(source_id) => {
                        if let Some(source) = source_id.resolve() {
                            info!("Run harvester");
                            crate::roles::local::harvester::harvest(&creep, creepmem, source)
                        } else {
                            creepmem.work = None;
                        }
                    }
                    crate::memory::Task::Hauler(building_id) => {
                        if let Some(building) = building_id.resolve() {
                            info!("Run hauler");
                            crate::roles::local::hauler::haul(&creep, creepmem, building)
                        }
                    }
                    crate::memory::Task::Rename(_) => {},
                }
            }
        }
    }
}
