use screeps::{Room, find};

use super::cache::RoomCache;

pub fn run_towers(room: &Room, cache: &RoomCache) {
    let towers = cache.structures.towers.values();
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