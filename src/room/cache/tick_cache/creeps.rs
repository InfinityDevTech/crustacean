use std::collections::HashMap;

use screeps::{find, game, Creep, Room, SharedCreepProperties};

use crate::{
    config::ALLIES,
    memory::{Role, ScreepsMemory},
    utils::{self, name_to_role},
};

#[derive(Debug, Clone)]
pub struct CreepCache {
    pub creeps_in_room: HashMap<String, Creep>,
    pub owned_creeps: HashMap<String, Creep>,
    pub creeps_of_role: HashMap<Role, Vec<String>>,

    pub enemy_creeps: Vec<Creep>,
    pub allied_creeps: Vec<Creep>,
}

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
            } else if ALLIES.contains(&creep.owner().username().as_str()) {
                self.allied_creeps.push(creep);
            } else {
                self.enemy_creeps.push(creep);
            }
        }

        if let Some(room_memory) = memory.rooms.get(&room.name()) {
            for creep_name in &room_memory.creeps {
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
                }
            }
        }
    }
}
