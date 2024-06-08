use screeps::{HasId, ResourceType, Room};

use crate::utils::scale_haul_priority;

use super::cache::tick_cache::{hauling::{HaulingPriority, HaulingType}, CachedRoom, RoomCache};

pub fn run_towers(cached_room: &mut CachedRoom) {
    for tower in cached_room.structures.towers.values() {
        if tower.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
            let priority = scale_haul_priority(
                tower.store().get_capacity(Some(ResourceType::Energy)),
                tower.store().get_used_capacity(Some(ResourceType::Energy)),
                HaulingPriority::Combat,
                true,
            );

            cached_room.hauling.create_order(
                tower.raw_id(),
                Some(ResourceType::Energy),
                    Some(tower.store().get_free_capacity(Some(ResourceType::Energy)) as u32),
                priority,
                HaulingType::Transfer,
            );
        }
        // Use cache here
        let enemies = &cached_room.creeps.enemy_creeps;
        if enemies.is_empty() {
            return;
        }
        let _ = tower.attack(enemies.first().unwrap());
    }
}
