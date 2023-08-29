use log::info;
use screeps::{game, Room};

use crate::{memory::ScreepsMemory, traits::room::RoomExtensions};

pub fn pre_market(room: &Room, creeps: Vec<String>, memory: &mut ScreepsMemory) {
    let starting_cpu = game::cpu::get_used();
    info!("[INDUSTRIES] Running construction industry");

    for name in creeps {
        let creepmem = memory.get_creep(&name);
        if let Some(task) = &creepmem.t {
            let creep = game::creeps().get(name).unwrap();
            match task {
                crate::memory::Task::Upgrader(controller_id) => {
                    if let Some(building) = controller_id.resolve() {
                        crate::roles::local::upgrader::upgrade(&creep, creepmem, building)
                    }
                }
                crate::memory::Task::Builder() => {
                    crate::roles::local::builder::run_creep(&creep, creepmem);
                }
                _ => {}
            }
        }
    }

    memory.stats.rooms.get_mut(&room.name_str()).unwrap().construction += game::cpu::get_used() - starting_cpu;
}
