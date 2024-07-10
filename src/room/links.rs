use std::cmp;

use screeps::{ResourceType, Room};

use super::cache::tick_cache::CachedRoom;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn balance_links(_room: &Room, room_cache: &mut CachedRoom) {
    if let Some(source_link) = &room_cache.structures.links.source {
        for link in source_link {
            if let Some(storage_link) = &room_cache.structures.links.storage {
                let storage_capacity = storage_link.store().get_free_capacity(Some(screeps::constants::ResourceType::Energy));
                let source_capacity = link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));

                if storage_capacity > 0 && source_capacity > 0 {
                    let transfer_amount = cmp::min(storage_link.store().get_used_capacity(Some(ResourceType::Energy)), link.store().get_used_capacity(Some(ResourceType::Energy)));
                    let _ = link.transfer_energy(storage_link, Some(transfer_amount));
                }
            }

            if let Some(fastfill_link) = &room_cache.structures.links.fast_filler {
                let fastfill_capacity = fastfill_link.store().get_free_capacity(Some(screeps::constants::ResourceType::Energy));
                let source_capacity = link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));

                if fastfill_capacity > 0 && source_capacity > 0 {
                    let transfer_amount = cmp::min(fastfill_link.store().get_used_capacity(Some(ResourceType::Energy)), link.store().get_used_capacity(Some(ResourceType::Energy)));
                    let _ = link.transfer_energy(fastfill_link, Some(transfer_amount));
                }
            }
        }
    }

    if let Some(storage_link) = &room_cache.structures.links.storage {
        if let Some(fastfill_link) = &room_cache.structures.links.fast_filler {
            let fastfill_capacity = fastfill_link.store().get_free_capacity(Some(screeps::constants::ResourceType::Energy));
            let source_capacity = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));

            if fastfill_capacity > 0 && source_capacity > 0 {
                let transfer_amount = cmp::min(fastfill_link.store().get_used_capacity(Some(ResourceType::Energy)), storage_link.store().get_used_capacity(Some(ResourceType::Energy)));
                let _ = storage_link.transfer_energy(fastfill_link, Some(transfer_amount));
            }
        }

        if let Some(controller_link) = &room_cache.structures.links.controller {
            let half_capacity = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy)) / 2;
            if controller_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy)) < half_capacity {
                let transfer_amount = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));
                let _ = storage_link.transfer_energy(controller_link, Some(transfer_amount));
            }
        }
    }
}