use std::collections::HashMap;

use log::info;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand::prelude::SliceRandom;
use screeps::{find, game, look, Creep, Direction, HasPosition, Part, Room, RoomName, SharedCreepProperties, SpawnOptions, StructureSpawn};

use crate::room::cache::tick_cache::RoomCache;
use crate::utils::get_body_cost;
use crate::{memory::{CreepMemory, Role, ScreepsMemory}, movement::utils::{dir_to_coords, num_to_dir}, room::cache::tick_cache::CachedRoom, utils::{name_to_role, role_to_name}};

use super::creep_sizing::{base_hauler_body, hauler_body, miner_body, repairer_body, upgrader_body};
use super::{base_hauler, fast_filler, get_required_role_counts, hauler, miner, repairer, scout, upgrader};

pub struct SpawnRequest {
    role: Role,
    body: Vec<Part>,
    priority: f64,
    cost: u32,

    destination_room: Option<RoomName>,

    creep_memory: CreepMemory,

    spawn_options: Option<SpawnOptions>
}

pub struct SpawnManager {
    room_spawn_queue: HashMap<RoomName, Vec<SpawnRequest>>,
    global_spawn_queue: Vec<SpawnRequest>,
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl SpawnManager {
    pub fn new() -> Self {
        Self {
            room_spawn_queue: HashMap::new(),
            global_spawn_queue: Vec::new(),
        }
    }

    pub fn create_room_spawn_request(&mut self, role: Role, body: Vec<Part>, priority: f64, cost: u32, owning_room: RoomName, creep_memory: Option<CreepMemory>, spawn_options: Option<SpawnOptions>) -> SpawnRequest {
        let mut creep_memory = if let Some(creep_memory) = creep_memory {
            creep_memory
        } else {
            CreepMemory {
                owning_room,
                ..Default::default()
            }
        };

        if creep_memory.role != role {
            creep_memory.role = role;
        }


        let request = SpawnRequest {
            role,
            body,
            priority,
            cost,

            destination_room: None,
            creep_memory,

            spawn_options
        };

        //if let Some(room_queue) = self.room_spawn_queue.get_mut(&owning_room) {
        //    room_queue.push(request);
        //} else {
        //    self.room_spawn_queue.insert(owning_room, vec![request.clone()]);
        //};


        request
    }

    pub fn run_spawning(&mut self, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
        for room in &cache.my_rooms {
            let room = game::rooms().get(*room).unwrap();
            let hauler_size = hauler(&room, cache, self, memory);
            let room_cache = cache.rooms.get(&room.name()).unwrap();

            let (active_spawns, inactive_spawns) = room_cache.structures.get_spawns();
            let mut spawned_count = 0;
            if active_spawns.is_empty() { continue; }

            let required_roles = get_required_role_counts(room_cache);
            let mut role_keys = required_roles.keys().collect::<Vec<_>>();

            // Sort the roles by their u8 vaules ascending
            role_keys.sort_by(|a, b| {
                let a = **a as u8;
                let b = **b as u8;

                a.cmp(&b)
            });

            for role in role_keys {
                let required_count_for_role = required_roles.get(role).unwrap();
                let current_count_for_role = room_cache.creeps.creeps_of_role.get(role).unwrap_or(&Vec::new()).len();

                if current_count_for_role < (*required_count_for_role).try_into().unwrap() {
                    let spawn_request = match role {
                        Role::Miner => miner(&room, room_cache, self),
                        Role::Hauler => hauler(&room, cache, self, memory),
                        Role::FastFiller => fast_filler(&room, room_cache, self),
                        Role::BaseHauler => base_hauler(&room, room_cache, self),
                        Role::Upgrader => upgrader(&room, room_cache, self),
                        Role::Repairer => repairer(&room, room_cache, self),
                        Role::Scout => scout(&room, room_cache, self),
                        _ => continue,
                    };

                    if let Some(spawn_request) = spawn_request {
                        let cost = get_body_cost(&spawn_request.body);
                        let can_spawn = self.can_room_spawn_creep(&room, room_cache, &spawn_request);

                        if can_spawn {
                            self.room_spawn_creep(&room, memory, room_cache, &spawn_request);
                        }
                    }
                }
            }
        }
    }

    pub fn clear_out_spawn_area(&self, room_cache: &mut CachedRoom) {
        for spawn in room_cache.structures.spawns.values() {
            let mut creeps_in_range = Vec::new();

            for creep in room_cache.creeps.owned_creeps.values() {
                if creep.pos().is_near_to(spawn.pos()) {
                    creeps_in_range.push(creep);
                }
            }

            let mut rng = StdRng::seed_from_u64(game::time() as u64);
            for creep in creeps_in_range {
                let name = creep.name();
                let role = name_to_role(&name);

                if role != Some(Role::FastFiller) {
                    let dir = num_to_dir(rng.gen_range(1..9) as u8);
                    dfs_clear_spawn(&creep, dir);
                }
            }
        }
    }

    pub fn room_spawn_creep(&self, room: &Room, memory: &mut ScreepsMemory, room_cache: &CachedRoom, request: &SpawnRequest) {
        let (available_spawn, _unavailable_spawns) = room_cache.structures.get_spawns();
        if available_spawn.is_empty() {
            return;
        }

        let options = if request.spawn_options.is_some() {
            let opts = &request.spawn_options;
            opts.as_ref().unwrap()
        } else {
            &SpawnOptions::default()
        };

        let name = format!("{}-{}-{}", role_to_name(request.role), room.name(), memory.get_id());

        if let Some(spawn) = available_spawn.first() {
            let spawn_result = spawn.spawn_creep_with_options(&request.body, &name, &options);

            if spawn_result.is_ok() {
                memory.create_creep(&room.name(), &name, request.creep_memory.clone());
            }
        }
    }

    pub fn can_room_spawn_creep(&self, room: &Room, room_cache: &CachedRoom, request: &SpawnRequest) -> bool {
        let cost = request.cost;

        if room.energy_available() < cost {
            return false;
        }

        let (available_spawn, _unavailable_spawns) = room_cache.structures.get_spawns();
        if available_spawn.is_empty() {
            return false;
        }

        let options = SpawnOptions::default().dry_run(true);
        let dry_run_result = available_spawn.first().unwrap().spawn_creep_with_options(&request.body, "dry_run", &options);

        dry_run_result.is_ok()
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn dfs_clear_spawn(creep: &Creep, dir: Direction) {
    let cur_x = creep.pos().x().u8();
    let cur_y = creep.pos().y().u8();

    let position = dir_to_coords(dir, cur_x, cur_y);

    let potential_creep = creep.room().unwrap().look_for_at_xy(look::CREEPS, position.0, position.1);

    if potential_creep.is_empty() {
        let _ = creep.move_direction(dir);
    } else {
        for creep in potential_creep {
            //let random_dir = num_to_dir(rng.gen_range(1..9) as u8);
            dfs_clear_spawn(&creep, dir);
        }
    }
}