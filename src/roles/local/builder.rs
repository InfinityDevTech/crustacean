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
        let movet = movement::move_target::MoveTarget {
            pos: site.pos(),
            range: 3
        }.find_path_to(creep.pos());
        creepmem.movement = Some(movet.clone());
        movement::creep::move_by_path(creep.name(), movet, creepmem);
    }
}
