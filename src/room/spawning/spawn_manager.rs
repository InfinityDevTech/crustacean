use std::collections::HashMap;

use log::info;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand::prelude::SliceRandom;
use screeps::{game, look, Creep, Direction, HasPosition, Part, Room, RoomName, SharedCreepProperties, SpawnOptions};

use crate::room::cache::tick_cache::RoomCache;
use crate::utils::get_unique_id;
use crate::{memory::{CreepMemory, Role, ScreepsMemory}, movement::utils::{dir_to_coords, num_to_dir}, room::cache::tick_cache::CachedRoom, utils::{name_to_role, role_to_name}};

use super::{base_hauler, create_spawn_requests_for_room, fast_filler, get_required_role_counts, hauler, miner, repairer, scout, upgrader};

pub struct SpawnRequest {
    name: Option<String>,
    role: Role,
    body: Vec<Part>,
    priority: f64,
    cost: u32,

    destination_room: Option<RoomName>,

    creep_memory: CreepMemory,

    spawn_options: Option<SpawnOptions>
}

pub struct SpawnManager {
    pub room_spawn_queue: HashMap<RoomName, Vec<SpawnRequest>>,
    pub global_spawn_queue: Vec<SpawnRequest>,
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl SpawnManager {
    pub fn new() -> Self {
        Self {
            room_spawn_queue: HashMap::new(),
            global_spawn_queue: Vec::new(),
        }
    }

    pub fn create_room_spawn_request(&self, role: Role, body: Vec<Part>, priority: f64, cost: u32, owning_room: RoomName, creep_memory: Option<CreepMemory>, spawn_options: Option<SpawnOptions>, name: Option<String>) -> SpawnRequest {
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

        //if let Some(room_queue) = self.room_spawn_queue.get_mut(&owning_room) {
        //    room_queue.push(request);
        //} else {
        //    self.room_spawn_queue.insert(owning_room, vec![request.clone()]);
        //};


        SpawnRequest {
            name,
            role,
            body,
            priority,
            cost,

            destination_room: None,
            creep_memory,

            spawn_options
        }
    }

    pub fn clear_out_spawn_area(&self, room_cache: &CachedRoom) {
        for spawn in room_cache.structures.spawns.values() {
            let mut creeps_in_range = Vec::new();

            // Only push when there is an imminent creep, because we
            // don't want to move creeps that are not in the way
            if spawn.spawning().is_none() {
                continue;
            }

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
                    dfs_clear_spawn(creep, dir);
                }
            }
        }
    }

    pub fn room_spawn_creep(&self, room: &Room, memory: &mut ScreepsMemory, room_cache: &CachedRoom, request: &SpawnRequest) -> bool {
        let (available_spawn, _unavailable_spawns) = room_cache.structures.get_spawns();
        if available_spawn.is_empty() {
            return false;
        }

        let options = if request.spawn_options.is_some() {
            let opts = &request.spawn_options;
            opts.as_ref().unwrap()
        } else {
            &SpawnOptions::default()
        };

        let name = if request.name.is_some() {
            request.name.as_ref().unwrap().clone()
        } else {
            format!("{}-{}-{}", role_to_name(request.role), room.name(), get_unique_id())
        };

        if let Some(spawn) = available_spawn.first() {
            let spawn_result = spawn.spawn_creep_with_options(&request.body, &name, options);

            if spawn_result.is_ok() {
                memory.create_creep(&room.name(), &name, request.creep_memory.clone());
                return true;
            }
        }

        false
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

pub fn run_spawning(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    for room in &cache.my_rooms.clone() {
        let room = game::rooms().get(*room).unwrap();
        let room_cache = cache.rooms.get(&room.name()).unwrap();

        let (active_spawns, inactive_spawns) = room_cache.structures.get_spawns();
        let mut spawned_this_tick = false;
        if active_spawns.is_empty() { continue; }

        if game::time() % 10 == 0 && !inactive_spawns.is_empty() {
            cache.spawning.clear_out_spawn_area(room_cache);
        }

        let required_roles = get_required_role_counts(room_cache);
        let mut required_role_keys = required_roles.keys().collect::<Vec<_>>();

        // Sort the roles by their u8 vaules ascending
        required_role_keys.sort_by(|a, b| {
            let a = **a as u8;
            let b = **b as u8;

            a.cmp(&b)
        });

        for required_role in required_role_keys {
            let required_count_for_role = required_roles.get(required_role).unwrap();
            let current_count_for_role = room_cache.creeps.creeps_of_role.get(required_role).unwrap_or(&Vec::new()).len();

            if current_count_for_role < (*required_count_for_role).try_into().unwrap() {
                let spawn_request = match required_role {
                    Role::Harvester => miner(&room, room_cache, &mut cache.spawning),
                    Role::Hauler => hauler(&room, cache, memory),
                    Role::FastFiller => fast_filler(&room, room_cache, &mut cache.spawning),
                    Role::BaseHauler => base_hauler(&room, room_cache, &mut cache.spawning),
                    Role::Upgrader => upgrader(&room, room_cache, &mut cache.spawning),
                    Role::Repairer => repairer(&room, room_cache, &mut cache.spawning),
                    Role::Scout => scout(&room, room_cache, &mut cache.spawning),
                    _ => continue,
                };

                if spawn_request.is_none() {
                    continue;
                }

                info!("[{}] {} Did not meet required count for role: {:#?}, spawning...", room.name(), required_role, required_count_for_role);

                if let Some(spawn_request) = spawn_request {
                    let can_spawn = cache.spawning.can_room_spawn_creep(&room, room_cache, &spawn_request);

                    if can_spawn {
                        let spawned = cache.spawning.room_spawn_creep(&room, memory, room_cache, &spawn_request);

                        if spawned {
                            spawned_this_tick = true;
                        }
                    }
                }
            }
        }

        if !spawned_this_tick {
            let mut room_requests = create_spawn_requests_for_room(&room, cache, memory);

            if let Some(other_room_requests) = cache.spawning.room_spawn_queue.get_mut(&room.name()) {
                room_requests.append(other_room_requests);
            }

            let room_cache = cache.rooms.get(&room.name()).unwrap();

            room_requests.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

            let room_requests = randomize_top_priorities(room_requests);

            if let Some(request) = room_requests.first() {
                info!("[{}] Highest spawn scorer role: {} - score: {}", room.name(), request.role, request.priority);
                let can_spawn = cache.spawning.can_room_spawn_creep(&room, room_cache, request);

                if can_spawn {
                    let spawned = cache.spawning.room_spawn_creep(&room, memory, room_cache, request);

                    if spawned {
                        spawned_this_tick = true;

                        break;
                    }
                }
            }
        }
    }
}

fn randomize_top_priorities(requests: Vec<SpawnRequest>) -> Vec<SpawnRequest> {
    let mut top_scorers = Vec::new();
    let top_scorer = requests.first().unwrap().priority;

    for request in requests {
        if request.priority == top_scorer {
            top_scorers.push(request);
        }
    }

    top_scorers.shuffle(&mut StdRng::seed_from_u64(game::time() as u64));

    top_scorers
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