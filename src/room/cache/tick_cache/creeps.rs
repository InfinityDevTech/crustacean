use std::collections::HashMap;

use screeps::{find, Creep, Room, SharedCreepProperties};

use crate::{config::ALLIES, memory::{Role, ScreepsMemory}, utils};

#[derive(Debug, Clone)]
pub struct CreepCache {
    pub creeps: HashMap<String, Creep>,
    pub creeps_of_role: HashMap<Role, Vec<String>>,

    pub enemy_creeps: Vec<Creep>,
    pub allied_creeps: Vec<Creep>,
}

impl CreepCache {
    pub fn new_from_room(room: &Room, _memory: &mut ScreepsMemory) -> CreepCache {
        let mut cache = CreepCache {
            creeps: HashMap::new(),
            creeps_of_role: HashMap::new(),

            enemy_creeps: Vec::new(),
            allied_creeps: Vec::new(),
        };

        cache.refresh_creep_cache(room);
        cache
    }

    pub fn refresh_creep_cache(&mut self, room: &Room) {
        let creeps = room.find(find::CREEPS, None);

        for creep in creeps {
            if creep.my() {
                let role = utils::name_to_role(&creep.name());
                if role.is_none() { continue; }

                if let Some(role_vec) = self.creeps_of_role.get_mut(&role.unwrap()) {
                    role_vec.push(creep.name());
                } else {
                    self.creeps_of_role.insert(role.unwrap(), vec![creep.name()]);
                }

                self.creeps.insert(creep.name(), creep);
            } else if ALLIES.contains(&creep.owner().username().as_str()) {
                self.allied_creeps.push(creep);
            } else {
                self.enemy_creeps.push(creep);
            }
        }
    }
}
