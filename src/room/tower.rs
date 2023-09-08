use screeps::{Room, StructureTower};

use crate::cache::ScreepsCache;

pub fn run_towers(room: &Room, cache: &ScreepsCache) {
    let towers = &cache.room_specific.get(&room.name().to_string()).unwrap().towers;

    for tower_id in towers {
        let tower: StructureTower = tower_id.resolve().unwrap();
        let hostile = cache.room_specific.get(&room.name().to_string()).unwrap().enemy_creeps.first();
        if let Some(hostile_id) = hostile {
            let _ = tower.attack(&hostile_id.resolve().unwrap());
        }
    }
}