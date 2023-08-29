use log::info;
use screeps::{game, Room};

use crate::{memory::ScreepsMemory, traits::room::RoomExtensions};

pub fn pre_market(
    room: &Room,
    creeps: Vec<String>,
    memory: &mut ScreepsMemory,
) {
    let starting_cpu = game::cpu::get_used();
    info!("[INDUSTRIES] Running mining industry...");

    for name in creeps {
        let creepmem = memory.get_creep(&name);
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
                            crate::roles::local::hauler::run_creep(&creep, creepmem, building)
                        }
                    }
                    _ => {},
                }
        }
    }

    memory.stats.rooms.get_mut(&room.name_str()).unwrap().mining += game::cpu::get_used() - starting_cpu;
}
