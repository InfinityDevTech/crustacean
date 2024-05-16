use std::{collections::HashMap, str::FromStr};

use log::info;
use screeps::{
    find, game, look::{self, LookResult}, HasId, HasPosition, ObjectId, Part, Room, Terrain
};

use crate::{memory::{ScoutedSource, ScreepsMemory}, traits::room::RoomExtensions};

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    // Horray, i did it better.
    let roommem = memory.get_room(&room.name());

    if game::cpu::bucket() >= 100 {
        info!("Initialising room: {}", room.name().to_string());
        let sources = room.find(find::SOURCES, None);
        let mut mining_sources = HashMap::new();

        for source in sources {

            let x = source.pos().x().u8();
            let y = source.pos().y().u8();
            let areas = room.look_for_at_area(look::TERRAIN, y - 1, x - 1, y + 1, x + 1);
            let mut available_spots = 0;

            for area in areas {
                match area.look_result {
                    LookResult::Terrain(Terrain::Plain) => available_spots += 1,
                    LookResult::Terrain(Terrain::Swamp) => available_spots += 1,
                    _ => {}
                }
            }

            mining_sources.insert(
                source.id(),
                ScoutedSource {
                    assigned_creeps: 0,
                    mining_spots: available_spots as u8,
                },
            );
        }

        roommem.sources = mining_sources;
        roommem.init = true;
    } else {
        info!(
            "CPU bucket is too low to initialise room: {}",
            room.name().to_string()
        );
        return;
    }
}
