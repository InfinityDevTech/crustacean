use std::collections::HashMap;

use log::info;
use screeps::{find, game, Creep, HasPosition, Part, Room, RoomName, RoomXY, SharedCreepProperties};

use crate::{
    allies, constants::{self, HOSTILE_PARTS}, heap, memory::{Role, ScreepsMemory}, traits::intents_tracking::CreepExtensionsTracking, utils::{self, name_to_role}
};

use super::structures::RoomStructureCache;

#[derive(Debug, Clone)]
pub struct CreepCache {
    pub creeps_in_room: HashMap<String, Creep>,
    pub owned_creeps: HashMap<String, Creep>,
    pub creeps_of_role: HashMap<Role, Vec<String>>,

    pub enemy_creeps: Vec<Creep>,
    pub enemy_creeps_with_attack: Vec<Creep>,
    pub allied_creeps: Vec<Creep>,

    pub creeps_at_pos: HashMap<RoomXY, Creep>,
}

impl CreepCache {
    pub fn new_from_room(room: &Room, memory: &mut ScreepsMemory, structures: &RoomStructureCache, owning_room: Option<RoomName>) -> CreepCache {
        let mut cache = CreepCache {
            creeps_in_room: HashMap::new(),
            owned_creeps: HashMap::new(),
            creeps_of_role: HashMap::new(),

            enemy_creeps: Vec::new(),
            enemy_creeps_with_attack: Vec::new(),
            allied_creeps: Vec::new(),

            creeps_at_pos: HashMap::new(),
        };

        cache.refresh_creep_cache(memory, room, structures, owning_room);
        cache
    }

    pub fn creeps_of_role(&self, role: Role) -> u32 {
        self.creeps_of_role.get(&role).unwrap_or(&Vec::new()).len() as u32
    }

    pub fn refresh_creep_cache(&mut self, memory: &mut ScreepsMemory, room: &Room, structures: &RoomStructureCache, owning_room: Option<RoomName>) {
        let creeps = room.find(find::CREEPS, None);

        for creep in creeps {
            if creep.my() {
                self.creeps_at_pos.insert(creep.pos().xy(), creep.clone());

                self.creeps_in_room.insert(creep.name(), creep);
            } else if allies::is_ally(&creep.owner().username(), None) {
                self.allied_creeps.push(creep);
            } else {
                if creep.body().iter().any(|x| HOSTILE_PARTS.contains(&x.part())) {
                    self.enemy_creeps_with_attack.push(creep.clone());
                }

                self.enemy_creeps.push(creep);
            }
        }

        for spawn in structures.spawns.values() {
            let creeps = spawn.spawning();
            if let Some(creeps) = creeps {
                if let Some(creep) = game::creeps().get(creeps.name().into()) {
                    self.creeps_in_room.insert(creeps.name().into(), creep);
                }
            }
        }

        //let mut non_existant_creeps = Vec::new();

        // TODO: This can get very bad very fast. Each room iterating over all creeps in memory, each tick???
        // BADDDDD
        if let Some(room_memory) = memory.rooms.get_mut(&room.name()) {
            room_memory.income = 0;
            room_memory.expense = 0;

            for creep_name in &room_memory.creeps.clone() {
                let creep = game::creeps().get(creep_name.to_string());

                if let Some(creep) = creep {
                    let role = name_to_role(creep_name);
                    if role.is_none() {
                        log::error!("Creep {} has no role", creep_name);
                        let _ = creep.ITsuicide();
                        continue;
                    }
                    let role = role.unwrap();

                    if role == Role::Harvester || role == Role::RemoteHarvester {
                        let harvest_power = utils::get_part_count(&creep.body(), Some(Part::Work)) as u32;
                        let energy_income = harvest_power * constants::HARVEST_POWER as u32;

                        room_memory.income += energy_income;
                    }

                    if role == Role::Upgrader {
                        let upgrade_power = utils::get_part_count(&creep.body(), Some(Part::Work)) as u32;
                        let energy_income = upgrade_power * constants::UPGRADE_POWER as u32;

                        room_memory.expense += energy_income;
                    } else if role == Role::Builder {
                        let build_power = utils::get_part_count(&creep.body(), Some(Part::Work)) as u32;
                        let energy_income = build_power * constants::BUILD_POWER as u32;

                        room_memory.expense += energy_income;
                    } else if role == Role::Repairer {
                        let repair_power = utils::get_part_count(&creep.body(), Some(Part::Work)) as u32;
                        let energy_income = repair_power * constants::REPAIR_POWER as u32;

                        room_memory.expense += energy_income;
                    }

                    room_memory.expense += room_memory.avg_spawn_expense.ceil() as u32;

                    self.owned_creeps.insert(creep_name.to_string(), creep);

                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self.creeps_of_role.entry(role)
                    {
                        e.insert(vec![creep_name.to_string()]);
                    } else {
                        self.creeps_of_role
                            .get_mut(&role)
                            .unwrap()
                            .push(creep_name.to_string());
                    }
                } else {
                    let _ = memory.creeps.remove(creep_name);
                    heap().creeps.lock().unwrap().remove(creep_name);
                    room_memory
                        .creeps
                        .retain(|x| x != creep_name);
                    continue;
                }
            }

            if room_memory.rcl < 8 {
                room_memory.income /= 2;
            }

            if let Some(storage) = structures.storage.as_ref() {
                let storage = storage.store().get_used_capacity(Some(screeps::ResourceType::Energy));
                if storage > 30000 {
                    room_memory.income *= storage / 10000;
                }
            }
        }
    }
}
