use log::info;
use screeps::{game, Room, HasPosition, TextStyle};

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

    if game::cpu::bucket() > 500 {
    for room in game::rooms().values() {
        let controller = room.controller().unwrap().pos();
        let result = room.flood_fill(vec![(controller.x().u8(), controller.y().u8())]);
        for x in 0..=50 {
            for y in 0..=50 {
                let text = result.get(x, y);

                room.visual().text(x as f32, y as f32 + 0.25, format!("{}", text), Some(TextStyle::default().color("#ffffff").align(screeps::TextAlign::Center).font(0.5)));
            }
        }
    }
    }

    memory.stats.rooms.get_mut(&room.name_str()).unwrap().construction += game::cpu::get_used() - starting_cpu;
}
