use screeps::{find, look::{self, LookResult}, HasId, HasPosition, ObjectId, Room, Source, Terrain};

use crate::memory::{ScoutedSource, ScreepsMemory};

pub fn find_sources(room: &Room) -> Vec<ScoutedSource> {
    let mut planned_sources = Vec::new();

    let sources = room.find(find::SOURCES, None);

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

        planned_sources.push(ScoutedSource {
            id: source.id(),
            assigned_creeps: 0,
            max_creeps: available_spots,
            work_parts: 0,
        });
    }

    planned_sources
}