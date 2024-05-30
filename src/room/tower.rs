use screeps::{find, HasId, ResourceType, Room};

use super::cache::tick_cache::{hauling::HaulingType, RoomCache};

pub fn run_towers(room: &Room, cache: &mut RoomCache) {
    for tower in cache.structures.towers.values() {
        // Use cache here
        let enemies = room.find(find::HOSTILE_CREEPS, None);
        if enemies.is_empty() {
            return;
        }
        if (tower.store().get_used_capacity(Some(ResourceType::Energy)) as f32)
            < (tower.store().get_capacity(Some(ResourceType::Energy)) as f32 * 0.5)
        {
            cache.hauling.create_order(
                tower.raw_id(),
                ResourceType::Energy,
                tower.store().get_free_capacity(Some(ResourceType::Energy)) as u32,
                super::cache::tick_cache::hauling::HaulingPriority::Combat,
                HaulingType::Transfer,
            );
        }
        let _ = tower.attack(enemies.first().unwrap());
    }
}
