use std::collections::HashMap;

use log::*;
use movement::creep;
use screeps::ConstructionSite;
use screeps::{
    constants::{Part, ResourceType},
    find, game,
    local::ObjectId,
    objects::{Creep, Source, StructureController},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::memory::{CreepMemory, ScreepsMemory};

mod building;
mod logging;
mod memory;
mod movement;
mod roles;
mod room;

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
    Rename(ObjectId<StructureController>)
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    debug!("Loop starting! CPU: {}", game::cpu::get_used());
    let mut memory = ScreepsMemory::init_memory();

    let mut ran_creeps: Vec<String> = Vec::new();
    for name in game::creeps().keys() {
        let creep = game::creeps().get(name.clone());
        if creep.is_none() {
            // Remove creep that no longer exists from memory
            memory.creeps.remove(&name);
        } else {
            match memory.creeps.get_mut(&name) {
                Some(creepmem) => {
                    run_creep(&creep.unwrap(), creepmem);
                }
                None => {
                    // This creep is new, add it to memory
                    memory.create_creep(&name);
                }
            }
            ran_creeps.push(name.clone());
        }
    }

    // Remove creeps that have died from memory
    memory.creeps.retain(|name, _| ran_creeps.contains(name));

    let mut additional = 0;
    for spawn in game::spawns().values() {
        // Default body for now, will be sorted out later.
        let body = [Part::Move, Part::Move, Part::Carry, Part::Work];
        if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
            let name = format!("{}-{}", game::time(), additional);
            match spawn.spawn_creep(&body, &name) {
                Ok(()) => {
                    additional += 1;
                    memory.create_creep(&name);
                }
                Err(e) => warn!("Couldn't spawn: {:?}", e),
            }
        }
    }

    // Bot is finished, write the local copy of memory.
    // This should be only executed ONCE per tick, as it is quite expensive.
    memory.write_memory();

    info!("Done! Cpu used: {}", game::cpu::get_used());
}

fn run_creep(creep: &Creep, creepmem: &mut CreepMemory) {
    if creep.spawning() {
        return;
    }

    let target = &creepmem.work;
    match target {
        Some(target) => {
            let creep_target = target;
            match creep_target {
                CreepTarget::Upgrade(controller_id)
                    if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 =>
                {
                    if let Some(controller) = controller_id.resolve() {
                        roles::local::upgrader::upgrade(creep, creepmem, controller);
                    } else {
                        creepmem.set_work(None);
                    }
                }
                CreepTarget::Harvest(source_id)
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) > 0 =>
                {
                    if let Some(source) = source_id.resolve() {
                        roles::local::harvester::harvest(creep, creepmem, source);
                    } else {
                        creepmem.set_work(None);
                    }
                }
                CreepTarget::Build(site_id) => {
                    if let Some(site) = site_id.resolve() {
                        if creep.pos().is_near_to(site.pos()) {
                            roles::local::builder::build(creep, creepmem, site);
                        } else {
                            let _ = creep.move_to(&site);
                        }
                    } else {
                        creepmem.work = None;
                    }
                }
                CreepTarget::Rename(controller_id) => {
                    if let Some(controller) = controller_id.resolve() {
                        if creep.pos().is_near_to(controller.pos()) {
                            let _ = creep.sign_controller(&controller, "Ferris FTW!");
                            creepmem.work = None;
                        } else {
                            creep::move_to(&creep.name(), creepmem, controller.pos());
                        }
                    } else {
                        creepmem.work = None;
                    }
                }
                _ => {
                    creepmem.set_work(None);
                }
            };
        }
        None => {
            // Should never fail.
            let room = creep.room().unwrap();
            if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                creepmem.set_work(Some(CreepTarget::Upgrade(room.controller().unwrap().id())));
            } else if let Some(source) = room.find(find::SOURCES_ACTIVE, None).get(0) {
                creepmem.set_work(Some(CreepTarget::Harvest(source.id())));
            }
        }
    }
}

#[wasm_bindgen(js_name = red_button)]
pub fn big_red_button() {
    for creep in game::creeps().values() {
        let _ = creep.say("GOD WHY IS THIS HAPPENING???", true);
        let _ = creep.suicide();
    }
    for room in game::rooms().values() {
        if let Some(controller) = room.controller() {
                for structure in room.find(find::MY_STRUCTURES, None) {
                    let _ = structure.destroy();
                }
                for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                    let _ = csite.remove();
                }
                let _ = controller.unclaim();
        }
    }
    let mut memory = memory::ScreepsMemory::init_memory();
    memory.creeps = HashMap::new();
    memory.rooms = HashMap::new();
    memory.write_memory();
}
