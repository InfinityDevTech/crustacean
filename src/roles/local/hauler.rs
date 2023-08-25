use std::cmp::min;

use log::info;
use screeps::{
    find, Creep, HasPosition, ResourceType, SharedCreepProperties, Structure, StructureObject,
};

use crate::{memory::CreepMemory, movement};

pub fn haul(creep: &Creep, creepmem: &mut CreepMemory, deposit: Structure) {
    let name = creep.name();
    let inventory = creep.store();
    if inventory.get_free_capacity(None)
        > inventory.get_used_capacity(Some(ResourceType::Energy)) as i32
    {
        let closest_energy = creep
            .pos()
            .find_closest_by_path(find::DROPPED_RESOURCES, None);
        if let Some(energy) = closest_energy {
            if creep.pos().is_near_to(energy.clone().pos()) {
                let _ = creep.pickup(&energy);
            } else {
                movement::creep::move_to(&name, creepmem, energy.pos())
            }
        }
    } else {
        let structure_object = StructureObject::from(deposit);
        if let Some(structure) = structure_object.as_transferable() {
            if structure_object
                .as_has_store()
                .unwrap()
                .store()
                .get_free_capacity(Some(ResourceType::Energy))
                > 0
            {
                info!("Running haul.");
                if creep.pos().is_near_to(structure.pos()) {
                    let _ = creep.transfer(
                        structure,
                        ResourceType::Energy,
                        Some(min(
                            inventory.get_used_capacity(Some(ResourceType::Energy)),
                            structure_object
                                .as_has_store()
                                .unwrap()
                                .store()
                                .get_free_capacity(Some(ResourceType::Energy)) as u32,
                    )));
                } else {
                    movement::creep::move_to(&name, creepmem, structure.pos());
                }
            } else {
                info!("Running construction.");
                let csite = creep.pos().find_closest_by_range(find::CONSTRUCTION_SITES);
                if let Some(site) = csite {
                    if creep.pos().is_near_to(site.pos()) {
                        let _ = creep.build(&site);
                    } else {
                        movement::creep::move_to(&name, creepmem, site.pos())
                    }
                }
            }
        }
    }
}
