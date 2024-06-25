use screeps::{HasId, ResourceType, StructureProperties};

use crate::utils::scale_haul_priority;

use super::cache::tick_cache::{hauling::{HaulingPriority, HaulingType}, CachedRoom};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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
                Some(tower.structure_type()),
                Some(ResourceType::Energy),
                Some(tower.store().get_free_capacity(Some(ResourceType::Energy)) as u32),
                priority,
                HaulingType::Transfer,
            );
        }
        // Use cache here
        let enemies = &cached_room.creeps.enemy_creeps;
        if enemies.is_empty() {
            let friendlies = &cached_room.creeps.owned_creeps;
            let allies = &cached_room.creeps.allied_creeps;

            if !friendlies.is_empty() {
                let damaged = friendlies
                    .iter()
                    .filter(|c| c.1.hits() < c.1.hits_max())
                    .collect::<Vec<_>>();

                if !damaged.is_empty() {
                    let target = damaged.first().unwrap();
                    let _ = tower.heal(target.1);
                    continue;
                }
            }

            if !allies.is_empty() {
                let damaged = allies
                    .iter()
                    .filter(|c| c.hits() < c.hits_max())
                    .collect::<Vec<_>>();

                if !damaged.is_empty() {
                    let target = damaged.first().unwrap();
                    let _ = tower.heal(*target);
                    continue;
                }
            }
        } else {
            let _ = tower.attack(enemies.first().unwrap());
        }
    }
}
