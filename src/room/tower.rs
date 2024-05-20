use screeps::{Room, find};

use super::structure_cache::RoomStructureCache;

pub fn run_towers(room: &Room, structure_cache: &RoomStructureCache) {
    let towers = structure_cache.towers.values();
    if towers.clone().count() > 0 {
        let enemies = room.find(find::HOSTILE_CREEPS, None);
        if enemies.is_empty() {
            return;
        }
        for tower in towers {
            let _ = tower.attack(enemies.first().unwrap());
        }
    }
}