use screeps::{HasHits, HasId, ResourceType, StructureProperties};

use crate::{
    memory::Role, traits::intents_tracking::TowerExtensionsTracking, utils::scale_haul_priority,
};

use super::cache::{
        hauling::{HaulingPriority, HaulingType},
        CachedRoom,
    };

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_towers(cached_room: &mut CachedRoom) {
    for tower in cached_room.structures.towers.values() {
        if tower.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
            let base_hauler_count = cached_room
                .creeps
                .creeps_of_role
                .get(&Role::BaseHauler)
                .unwrap_or(&Vec::new())
                .len();

            let mut storage_blocked = false;

            if let Some(_storage) = &cached_room.structures.storage {
                storage_blocked = base_hauler_count < 1;
            }

            if !storage_blocked {
                let mut priority = scale_haul_priority(
                    tower.store().get_capacity(Some(ResourceType::Energy)),
                    tower.store().get_used_capacity(Some(ResourceType::Energy)),
                    HaulingPriority::Combat,
                    true,
                );

                if tower.store().get_used_capacity(Some(ResourceType::Energy)) < 100 {
                    priority -= 6.0;

                    cached_room.hauling.create_order(
                        tower.raw_id(),
                        Some(tower.structure_type()),
                        Some(ResourceType::Energy),
                        Some(tower.store().get_free_capacity(Some(ResourceType::Energy)) as u32),
                        priority,
                        HaulingType::NoDistanceCalcTransfer,
                    );

                    continue;
                } else {
                    priority += 1.0;

                    cached_room.hauling.create_order(
                        tower.raw_id(),
                        Some(tower.structure_type()),
                        Some(ResourceType::Energy),
                        Some(tower.store().get_free_capacity(Some(ResourceType::Energy)) as u32),
                        priority,
                        HaulingType::NoDistanceCalcTransfer,
                    );
                }
            }
        }
        // Use cache here
        let enemies = &cached_room.creeps.enemy_creeps;
        if enemies.is_empty() {
            let friendlies = &cached_room.creeps.creeps_in_room;
            let allies = &cached_room.creeps.allied_creeps;

            if !friendlies.is_empty() {
                let damaged = friendlies
                    .iter()
                    // I cant believe it took me this long to find.
                    // It wond repair shit of there is a creep spawning.
                    .filter(|c| c.1.hits() < c.1.hits_max() && !c.1.spawning())
                    .collect::<Vec<_>>();

                if !damaged.is_empty() {
                    let target = damaged.first().unwrap();
                    let _ = tower.ITheal(target.1);
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
                    let _ = tower.ITheal(*target);
                    continue;
                }
            }

            let ramparts = cached_room.structures.ramparts.clone();
            let mut lowest_hits = u32::MAX;
            let mut lowest_rampart = None;

            for rampart in ramparts {
                if rampart.hits() > 2000 {
                    continue;
                }

                if rampart.hits() < lowest_hits && rampart.hits() < 2000 {
                    lowest_hits = rampart.hits();
                    lowest_rampart = Some(rampart);
                }
            }

            if let Some(rampart) = lowest_rampart {
                let _ = tower.ITrepair(&rampart);
                continue;
            }
        } else {
            let _ = tower.ITattack(enemies.first().unwrap());
            continue;
        }
    }
}
