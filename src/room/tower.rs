use log::info;
use screeps::{HasHits, HasId, ResourceType, StructureProperties};

use crate::{traits::intents_tracking::TowerExtensionsTracking, utils::scale_haul_priority};

use super::cache::tick_cache::{hauling::{HaulingPriority, HaulingType}, CachedRoom};

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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
                    info!("Attempting to heal: {:?}", target.1);
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
                    info!("Attempting to heal ally: {:?}", target);
                    let _ = tower.ITheal(*target);
                    continue;
                }
            }

            let mut ramparts = cached_room.structures.ramparts.clone();
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
            return;
        }
    }
}
