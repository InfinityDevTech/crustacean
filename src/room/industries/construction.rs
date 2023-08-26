use log::info;
use screeps::{game, Room};

use crate::memory::ScreepsMemory;

pub fn pre_market(room: &Room, creeps: Vec<String>, memory: &mut ScreepsMemory) {
    info!("[INDUSTRIES] Running construction industry");

    for name in creeps {
        let creepmem = memory.creeps.get_mut(&name).unwrap();
        if let Some(task) = &creepmem.t {
            let creep = game::creeps().get(name).unwrap();
            match task {
                crate::memory::Task::Upgrader(controller_id) => {
                    if let Some(building) = controller_id.resolve() {
                        crate::roles::local::upgrader::upgrade(&creep, creepmem, building)
                    }
                }
                _ => {}
            }
        }
    }
}
