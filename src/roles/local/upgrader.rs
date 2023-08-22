use log::warn;
use screeps::{Creep, ErrorCode, SharedCreepProperties, StructureController, HasPosition};

use crate::{memory::CreepMemory, movement};

pub fn upgrade(creep: &Creep, creepmem: &mut CreepMemory, controller: StructureController) {
    let name = creep.name();
    creep
        .upgrade_controller(&controller)
        .unwrap_or_else(|e| match e {
            ErrorCode::NotInRange => {
                movement::creep::move_to(&name, creepmem, controller.pos());
            }
            _ => {
                let _ = creep.say("🚧", false);
                warn!("Error upgrading controller: {:?}", e);
            }
        });
}
