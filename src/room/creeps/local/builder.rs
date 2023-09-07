use screeps::{find, Creep, HasPosition, ResourceType};

use crate::{
    cache::{self, ScreepsCache},
    memory::CreepMemory,
    traits::{creep::CreepExtensions, room::RoomExtensions},
};

pub fn run_creep(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) {
    if creepmem.s == "energy" {
        find_energy(creep, creepmem, cache);
    } else if creepmem.s == "work" && !build(creep, creepmem, cache) {
        repair(creep, creepmem, cache);
    }
}

pub fn build(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) -> bool {
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "energy".to_string();
        find_energy(creep, creepmem, cache);
        return true;
    }
    if let Some(csites) = cache.room_specific.get(&creep.room().unwrap().name().to_string()) {
        if let Some(site_id) = csites.csites.first() {
            let site = site_id.resolve().unwrap();
            if creep.better_is_near(site.clone().pos()) <= 1 {
                let _ = creep.build(&site);
                return true;
            } else {
                creep.better_move_to(creepmem, cache, site.pos(), 1);
                return true;
            }
        }
        false
    } else {
        false
    }
}

pub fn repair(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) {
    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "energy".to_string();
        find_energy(creep, creepmem, cache);
        return;
    }
    let closest_site = creep.room().unwrap().find(find::STRUCTURES, None);
    for csite in closest_site {
        if let Some(attackable) = csite.as_attackable() {
            if attackable.hits() < attackable.hits_max() {
                match csite.as_structure().structure_type() {
                    screeps::StructureType::Wall => {
                        if attackable.hits() > 100000 {
                            continue;
                        }
                    }
                    screeps::StructureType::Rampart => {
                        if attackable.hits() > 100000 {
                            continue;
                        }
                    }
                    _ => {}
                }
                if creep.pos().is_near_to(attackable.pos()) {
                    let _ = creep.repair(csite.as_structure());
                    break;
                } else {
                    creep.better_move_to(creepmem, cache, attackable.pos(), 1);
                    break;
                }
            } else {
                continue;
            }
        }
    }
}

pub fn find_energy(creep: &Creep, creepmem: &mut CreepMemory, cache: &mut ScreepsCache) {
    let closest_energy = cache
        .room_specific.get(&creep.room().unwrap().name().to_string()).unwrap().energy.first();
    if let Some(energy_id) = closest_energy {
        let energy = energy_id.resolve().unwrap();
        if creep.better_is_near(energy.clone().pos()) <= 1{
            let _ = creep.pickup(&energy);
        } else {
            creep.better_move_to(creepmem, cache, energy.pos(), 1)
        }
    }
    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
        creepmem.s = "work".to_string();
        build(creep, creepmem, cache);
    }
}
