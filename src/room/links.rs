use std::cmp;

use screeps::{game, HasId, ResourceType, Room, StructureType};

use crate::memory::Role;

use super::cache::{hauling::HaulingType, CachedRoom};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn balance_links(_room: &Room, room_cache: &mut CachedRoom) {
    if game::cpu::bucket() < 100 {
        return;
    }

    if let Some(source_link) = &room_cache.structures.links().source {
        for link in source_link {
            if let Some(storage_link) = &room_cache.structures.links().storage {
                let storage_capacity = storage_link.store().get_free_capacity(Some(screeps::constants::ResourceType::Energy));
                let source_capacity = link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));

                if storage_capacity > 0 && source_capacity > 0 {
                    let transfer_amount = cmp::min(storage_link.store().get_used_capacity(Some(ResourceType::Energy)), link.store().get_used_capacity(Some(ResourceType::Energy)));
                    let _ = link.transfer_energy(storage_link, Some(transfer_amount));
                }
            }

            if let Some(fastfill_link) = &room_cache.structures.links().fast_filler {
                let fastfill_capacity = fastfill_link.store().get_free_capacity(Some(screeps::constants::ResourceType::Energy));
                let source_capacity = link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));

                if fastfill_capacity > 0 && source_capacity > 0 {
                    let transfer_amount = cmp::min(fastfill_link.store().get_used_capacity(Some(ResourceType::Energy)), link.store().get_used_capacity(Some(ResourceType::Energy)));
                    let _ = link.transfer_energy(fastfill_link, Some(transfer_amount));
                }
            }

            if let Some(controller_link) = &room_cache.structures.links().controller {
                let controller_capacity = controller_link.store().get_free_capacity(Some(ResourceType::Energy));
                let source_capacity = link.store().get_used_capacity(Some(ResourceType::Energy));

                if controller_capacity > 0 && source_capacity > 0 {
                    let transfer_amount = cmp::min(controller_link.store().get_used_capacity(Some(ResourceType::Energy)), link.store().get_used_capacity(Some(ResourceType::Energy)));
                    let _ = link.transfer_energy(controller_link, Some(transfer_amount));
                }
            }
        }
    }

    if let Some(storage_link) = &room_cache.structures.links().storage {
        let base_hauler_count = room_cache.creeps.creeps_of_role(Role::BaseHauler);

        if base_hauler_count == 0 {
            let amount = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));

            room_cache.hauling.create_order(storage_link.raw_id(), Some(StructureType::Extension), Some(ResourceType::Energy), Some(amount), -(amount as f32), HaulingType::Offer);
        }

        if let Some(fastfill_link) = &room_cache.structures.links().fast_filler {
            let fastfill_capacity = fastfill_link.store().get_free_capacity(Some(screeps::constants::ResourceType::Energy));
            let source_capacity = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));

            if fastfill_capacity > 0 && source_capacity > 0 {
                let transfer_amount = cmp::min(fastfill_link.store().get_used_capacity(Some(ResourceType::Energy)), storage_link.store().get_used_capacity(Some(ResourceType::Energy)));
                let _ = storage_link.transfer_energy(fastfill_link, Some(transfer_amount));
            }
        }

        if let Some(controller_link) = &room_cache.structures.links().controller {
            let half_capacity = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy)) / 2;
            if controller_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy)) < half_capacity {
                let transfer_amount = storage_link.store().get_used_capacity(Some(screeps::constants::ResourceType::Energy));
                let _ = storage_link.transfer_energy(controller_link, Some(transfer_amount));
            }
        }
    }
}