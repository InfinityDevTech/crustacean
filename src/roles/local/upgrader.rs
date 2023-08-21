use log::warn;
use screeps::{Creep, ErrorCode, SharedCreepProperties, StructureController, HasPosition};

use crate::memory::CreepMemory;

pub fn upgrade(creep: &Creep, creepmem: &mut CreepMemory, controller: StructureController) {
    let name = creep.name();
    creep
        .upgrade_controller(&controller)
        .unwrap_or_else(|e| match e {
            ErrorCode::NotInRange => {
                match &creepmem.movement {
                    Some(path) => {
                        crate::movement::creep::move_by_path(name.clone(), path.clone(), creepmem);
                    }
                    None => {
                        let path = crate::movement::move_target::MoveTarget {
                            pos: controller.pos(),
                            range: 1
                        }
                        .find_path_to(creep.pos());
                        creepmem.movement = Some(path.clone());
                        crate::movement::creep::move_by_path(name.clone(), path, creepmem);
                    }
                };
            }
            _ => {
                let _ = creep.say("🚧", false);
                warn!("Error upgrading controller: {:?}", e);
            }
        });
}
