use screeps::{Creep, HasPosition, Position, RoomCoordinate, SharedCreepProperties};

use crate::{memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_physical_observer(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if let Some(creep_memory) = memory.creeps.get_mut(&creep.name()) {
        if let Some(target_room) = creep_memory.scout_target {
            let creep_room = creep.room().unwrap();

            let pos = Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(25).unwrap(), target_room);

            if creep_room.name() != target_room {
                let _ = creep.say(&format!("🔍 {}", target_room), true);
                creep.better_move_to(
                    creep_memory,
                    cache.rooms.get_mut(&creep_room.name()).unwrap(),
                    Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(25).unwrap(), target_room),
                    23,
                    Default::default(),
                );
            } else {
                if creep.pos().get_range_to(pos) > 23 {
                    creep.better_move_to(
                        creep_memory,
                        cache.rooms.get_mut(&creep_room.name()).unwrap(),
                        Position::new(RoomCoordinate::new(25).unwrap(), RoomCoordinate::new(25).unwrap(), target_room),
                        23,
                        Default::default(),
                    );
                }
                let _ = creep.say("👁️ 🫵", true);
            }
        }
    }
}