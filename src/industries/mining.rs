use std::collections::HashMap;

use screeps::game;

use crate::memory::{Careers, ScreepsMemory, CreepMemory};

pub fn pre_market(memory: &mut ScreepsMemory) -> &mut ScreepsMemory {
    // This is done as to not affect the actual memory.
    let creeps_iter = memory.creeps.clone().into_iter();
    let workers: HashMap<String, CreepMemory> = creeps_iter.filter(|(_, memory)| {
        memory.work.is_some() && memory.work.as_ref().unwrap().career == Careers::Mining
    }).collect();

    for (name, mut creepmem) in workers {
        let creep = game::creeps().get(name);
        let task = &creepmem.work.clone().unwrap().task;
        if creep.is_none() {continue};
        match task {
            Some(task) =>{
                match task {
                    crate::memory::Task::Miner(source_id) => {
                        if let Some(source) = source_id.resolve() {
                        crate::roles::local::harvester::harvest(&creep.unwrap(), &mut creepmem, source)
                        } else {
                            creepmem.work = None;
                        }
                    },
                    crate::memory::Task::Hauler(building_id) => {
                            if let Some(building) = building_id.resolve() {
                            crate::roles::local::hauler::haul(&creep.unwrap(), &mut creepmem, building)
                            } else {
                                creepmem.work = None;
                            }
                    },
                    _ => {
                        creepmem.work = None;
                    }
                }
            },
            None => todo!(),
        }
    }

    memory
}
