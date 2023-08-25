use log::info;
use screeps::{game, Room};

use crate::memory::ScreepsMemory;

pub fn pre_market(
    room: &Room,
    creeps: Vec<String>,
    memory: &mut ScreepsMemory,
) {
    let roommem = memory.rooms.get_mut(&room.name().to_string()).unwrap();
    info!("{}", creeps.len());

    for name in creeps {
        let creepmem = roommem.creeps.get_mut(&name).unwrap();
        if let Some(work) = &creepmem.work {
            if let Some(task) = &work.task {
                let creep = game::creeps().get(name).unwrap();
                match task {
                    crate::memory::Task::Upgrader(controller_id) => {
                        if let Some(controller) = controller_id.resolve() {
                            crate::roles::local::upgrader::upgrade(&creep, creepmem, controller)
                        } else {
                            creepmem.work = None;
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
