#![allow(dead_code)]
use screeps::{find, Creep, HasPosition, ResourceType};

use crate::{memory::CreepMemory, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory) {
    if creepmem.s == "energy" {
        find_energy(creep, creepmem);
    } else if creepmem.s == "work" && !build(creep, creepmem) {
        repair(creep, creepmem);
    }
}

pub fn build(creep: &Creep, creepmem: &mut CreepMemory) -> bool {
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "energy".to_string();
        find_energy(creep, creepmem);
        return true;
    }
    let closest_site = creep.pos().find_closest_by_range(find::CONSTRUCTION_SITES);
        if let Some(site) = closest_site {
            if creep.pos().is_near_to(site.clone().pos()) {
                let _ = creep.build(&site);
                return true;
            } else {
                creep.better_move_to(creepmem, site.pos(), 1);
                return true;
            }
        }
        false
}

pub fn repair(creep: &Creep, creepmem: &mut CreepMemory) {
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "energy".to_string();
        find_energy(creep, creepmem);
        return;
    }
    let closest_site = creep.room().unwrap().find(find::MY_STRUCTURES, None);
        for csite in closest_site {
            if let Some(attackable) = csite.as_attackable() {
                if attackable.hits() < attackable.hits_max() {
                    if creep.pos().is_near_to(attackable.pos()) {
                        let _ = creep.repair(csite.as_structure());
                        break;
                    } else {
                        creep.better_move_to(creepmem, attackable.pos(), 1);
                        break;
                    }
                }
            }
        }
}

pub fn find_energy(creep: &Creep, creepmem: &mut CreepMemory) {
    let closest_energy = creep
            .pos()
            .find_closest_by_range(find::DROPPED_RESOURCES);
        if let Some(energy) = closest_energy {
            if creep.pos().is_near_to(energy.clone().pos()) {
                let _ = creep.pickup(&energy);
            } else {
                creep.better_move_to(creepmem, energy.pos(), 1)
            }
        }
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            creepmem.s = "work".to_string();
            build(creep, creepmem);
        }
}
