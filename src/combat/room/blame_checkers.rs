use screeps::{find, game, Creep, HasPosition, ObjectId};

use crate::room::cache::tick_cache::RoomCache;

// Expensive function, iterates over all found creeps in the cache
// Checks within a range of 5
pub fn creep_attacked(creep_id: &ObjectId<Creep>, cache: &mut RoomCache) -> Option<String> {
    let creep = game::get_object_by_id_typed(creep_id);

    if let Some(creep) = creep {
        let hostile_creeps = cache.creeps.enemy_creeps.clone();

        for hostile_creep in hostile_creeps {
            if hostile_creep.pos().get_range_to(creep.pos()) < 5 {
                return Some(hostile_creep.owner().username());
            }
        }

        return None;
    }

    None
}