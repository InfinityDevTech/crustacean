use screeps::{find, HasId, ResourceType, Room};

use super::cache::{hauling::HaulingType, RoomCache};

pub fn run_towers(room: &Room, cache: &mut RoomCache) {
    let towers = cache.structures.towers.values();
    if towers.clone().count() > 0 {
        let enemies = room.find(find::HOSTILE_CREEPS, None);
        if enemies.is_empty() {
            return;
        }
        for tower in towers {
            if (tower.store().get_used_capacity(Some(ResourceType::Energy)) as f32) < (tower.store().get_capacity(Some(ResourceType::Energy)) as f32 * 0.5){
                cache.hauling.create_order(tower.raw_id(),
                ResourceType::Energy,
                tower.store().get_free_capacity(Some(ResourceType::Energy)) as u32,
                super::cache::hauling::HaulingPriority::Combat,
                HaulingType::Transfer
            );
            }
            let _ = tower.attack(enemies.first().unwrap());
        }
    }
}