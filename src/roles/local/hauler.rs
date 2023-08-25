use screeps::{Creep, SharedCreepProperties, HasPosition, find, ResourceType, Structure, StructureObject};

use crate::{memory::CreepMemory, movement};

pub fn haul(creep: &Creep, creepmem: &mut CreepMemory, deposit: Structure) {
    let name = creep.name();
    let inventory = creep.store();
    if inventory.get_free_capacity(None) > inventory.get_used_capacity(Some(ResourceType::Energy)) as i32 {
        let closest_energy = creep.pos().find_closest_by_path(find::DROPPED_RESOURCES, None);
        if let Some(energy) = closest_energy {
                if creep.pos().is_near_to(energy.clone().pos()) {
                    let _ = creep.pickup(&energy);
                } else {
                    movement::creep::move_to(&name, creepmem, energy.pos())
                }
            }
    } else {
        //let structure_object = StructureObject::from(deposit);
        //if let Some(structure) = structure_object.as_transferable() {
        //    let _ = creep.transfer(structure, ResourceType::Energy, Some(inventory.get_used_capacity(Some(ResourceType::Energy))));
        //}
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
