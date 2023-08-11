use log::*;
use screeps::ConstructionSite;
use screeps::{
    constants::{ErrorCode, Part, ResourceType},
    enums::StructureObject,
    find, game,
    local::ObjectId,
    objects::{Creep, Source, StructureController},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::memory::{CreepMemory, Memory};

mod logging;
mod memory;

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen(js_name = setup)]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

// this enum will represent a creep's lock on a specific target object, storing a js reference
// to the object id so that we can grab a fresh reference to the object each successive tick,
// since screeps game objects become 'stale' and shouldn't be used beyond the tick they were fetched
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum CreepTarget {
    Upgrade(ObjectId<StructureController>),
    Harvest(ObjectId<Source>),
    Build(ObjectId<ConstructionSite>),
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    debug!("Loop starting! CPU: {}", game::cpu::get_used());
    debug!("Getting memory");
    let mut memory = crate::Memory::init_memory();
    debug!("Running creeps");
    for (name, _) in memory.creeps.clone() {
        let creep = game::creeps().get(name.clone());
        if creep.is_none() {
            info!("Found non-existent creep, removing...");
            memory.creeps.remove(&name);
            memory.write_memory();
        } else {
            run_creep(&creep.unwrap(), &mut memory.creeps.get_mut(&name).unwrap());
        }
    }

    debug!("Running spawns");
    let mut additional = 0;
    for spawn in game::spawns().values() {
        debug!("Running spawn {}", String::from(spawn.name()));

        let body = [Part::Move, Part::Move, Part::Carry, Part::Work];
        if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
            // create a unique name, spawn.
            let name_base = game::time();
            let name = format!("{}-{}", name_base, additional);
            // note that this bot has a fatal flaw; spawning a creep
            // creates Memory.creeps[creep_name] which will build up forever;
            // these memory entries should be prevented (todo doc link on how) or cleaned up
            match spawn.spawn_creep(&body, &name) {
                Ok(()) => additional += 1,
                Err(e) => warn!("Couldn't spawn: {:?}", e),
            }
        }
    }

    memory.write_memory();
    info!("Done! cpu: {}", game::cpu::get_used());
}

fn run_creep(creep: &Creep, creepmem: &mut CreepMemory) {
    if creep.spawning() {
        return;
    }
    let name = creep.name();
    debug!("Running creep {}", name);

    let target = &creepmem.work;
    match target {
        Some(target) => {
            let creep_target = target;
            match creep_target {
                CreepTarget::Upgrade(controller_id)
                    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 =>
                {
                    if let Some(controller) = controller_id.resolve() {
                        creep
                            .upgrade_controller(&controller)
                            .unwrap_or_else(|e| match e {
                                ErrorCode::NotInRange => {
                                    let _ = creep.move_to(&controller);
                                }
                                _ => {
                                    warn!("Couldn't upgrade: {:?}", e);
                                    creepmem.work = None;
                                }
                            });
                    } else {
                        creepmem.work = None;
                    }
                }
                CreepTarget::Harvest(source_id)
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) > 0 =>
                {
                    if let Some(source) = source_id.resolve() {
                        if creep.pos().is_near_to(source.pos()) {
                            creep.harvest(&source).unwrap_or_else(|e| {
                                warn!("couldn't harvest: {:?}", e);
                                creepmem.work = None;
                            });
                        } else {
                            let _ = creep.move_to(&source);
                        }
                    } else {
                        creepmem.work = None;
                    }
                }
                CreepTarget::Build(site_id) => {
                    if let Some(site) = site_id.resolve() {
                        if creep.pos().is_near_to(site.pos()) {
                            creep.build(&site).unwrap_or_else(|e| {
                                warn!("couldn't build: {:?}", e);
                                creepmem.work = None;
                            });
                        } else {
                            let _ = creep.move_to(&site);
                        }
                    } else {
                        creepmem.work = None;
                    }
                }
                _ => {
                    creepmem.work = None;
                }
            };
        }
        None => {
            // no target, let's find one depending on if we have energy
            let room = creep.room().expect("couldn't resolve creep room");
            if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                for structure in room.find(find::STRUCTURES, None).iter() {
                    if let StructureObject::StructureController(controller) = structure {
                        creepmem.work = Some(CreepTarget::Upgrade(controller.id()));
                        break;
                    }
                }
            } else if let Some(source) = room.find(find::SOURCES_ACTIVE, None).get(0) {
                creepmem.work = Some(CreepTarget::Harvest(source.id()));
            }
        }
    }
}
