use std::collections::HashMap;

use log::info;
use screeps::{find, game, Creep, Room, SharedCreepProperties};

use crate::{
    config::ALLIES,
    memory::{Role, ScreepsMemory},
    utils::name_to_role,
};

#[derive(Debug, Clone)]
pub struct CreepCache {
    pub creeps_in_room: HashMap<String, Creep>,
    pub owned_creeps: HashMap<String, Creep>,
    pub creeps_of_role: HashMap<Role, Vec<String>>,

    pub enemy_creeps: Vec<Creep>,
    pub allied_creeps: Vec<Creep>,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CreepCache {
    pub fn new_from_room(room: &Room, memory: &mut ScreepsMemory) -> CreepCache {
        let mut cache = CreepCache {
            creeps_in_room: HashMap::new(),
            owned_creeps: HashMap::new(),
            creeps_of_role: HashMap::new(),

            enemy_creeps: Vec::new(),
            allied_creeps: Vec::new(),
        };

        cache.refresh_creep_cache(memory, room);
        cache
    }

    pub fn refresh_creep_cache(&mut self, memory: &mut ScreepsMemory, room: &Room) {
        let creeps = room.find(find::CREEPS, None);

        for creep in creeps {
            if creep.my() {
                self.creeps_in_room.insert(creep.name(), creep);
            } else if memory.allies.contains(&creep.owner().username()) {
                self.allied_creeps.push(creep);
            } else {
                self.enemy_creeps.push(creep);
            }
        }

        //let mut non_existant_creeps = Vec::new();

        // TODO: This can get very bad very fast. Each room iterating over all creeps in memory, each tick???
        // BADDDDD
        if let Some(room_memory) = memory.rooms.get_mut(&room.name()) {
            for creep_name in &room_memory.creeps.clone() {
                let creep = game::creeps().get(creep_name.to_string());

                if let Some(creep) = creep {
                    let role = name_to_role(creep_name);
                    if role.is_none() {
                        log::error!("Creep {} has no role", creep_name);
                        let _ = creep.suicide();
                        continue;
                    }

                    self.owned_creeps.insert(creep_name.to_string(), creep);

                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self.creeps_of_role.entry(role.unwrap())
                    {
                        e.insert(vec![creep_name.to_string()]);
                    } else {
                        self.creeps_of_role
                            .get_mut(&role.unwrap())
                            .unwrap()
                            .push(creep_name.to_string());
                    }
                } else {
                    let _ = memory.creeps.remove(creep_name);
                    room_memory
                        .creeps
                        .retain(|x| x != creep_name);
                    continue;
                }
            }
        }
    }
}
