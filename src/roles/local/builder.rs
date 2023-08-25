#![allow(dead_code)]
use log::warn;
use screeps::{ConstructionSite, Creep, HasPosition, SharedCreepProperties};

use crate::{memory::CreepMemory, movement};

pub fn build(creep: &Creep, creepmem: &mut CreepMemory, site: ConstructionSite) {
    if creep.pos().is_near_to(site.pos()) {
        creep.build(&site).unwrap_or_else(|e| {
            warn!("couldn't build: {:?}", e);
            creepmem.work = None;
        });
    } else {
        movement::creep::move_to(&creep.name(), creepmem, site.pos());
    }
}
