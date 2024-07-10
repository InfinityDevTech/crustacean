use std::collections::HashMap;

use log::info;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand::prelude::SliceRandom;
use screeps::{game, look, Creep, Direction, HasPosition, Part, Room, RoomName, SharedCreepProperties, SpawnOptions};

use crate::movement::move_target::{MoveOptions, MoveTarget};
use crate::room::cache::tick_cache::RoomCache;
use crate::traits::position::RoomXYExtensions;
use crate::utils::get_unique_id;
use crate::{memory::{CreepMemory, Role, ScreepsMemory}, movement::utils::{dir_to_coords, num_to_dir}, room::cache::tick_cache::CachedRoom, utils::{name_to_role, role_to_name}};

use super::{base_hauler, create_spawn_requests_for_room, fast_filler, get_required_role_counts, hauler, harvester, repairer, scout, upgrader};

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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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

        let body = if body.len() > 50 {
            info!("Body too large for {} {}/50 parts", role, body.len());
            body.iter().take(50).cloned().collect::<Vec<_>>() // Limit body to 50 parts
        } else {
            body
        };

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
            } else {
                info!("[{}] Failed to spawn {} creep: {:#?}", room.name(), request.role, spawn_result);
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

pub fn calculate_hauler_needs(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let room_memory = memory.rooms.get(&room.name()).unwrap().clone();
    let owning_cache = cache.rooms.get(&room.name()).unwrap();

    let body = crate::room::spawning::creep_sizing::hauler_body(room);
    let carry_count = body.iter().filter(|p| *p == &Part::Carry).count();

    let mut carry_requirement = 0;

    if room_memory.hauler_count == 0 || game::time() % 100 == 0 || room_memory.hauler_count > 200 {
        for remote in &room_memory.remotes {
            if game::rooms().get(*remote).is_some() {
                let room_cache = cache.rooms.get(remote).unwrap();

                for source in &room_cache.resources.sources {
                    let source_ept = (source.calculate_work_parts() * 2) as u128;
                    let source = source.source.clone();

                    let (out_steps, in_steps) = if let Some(storage) =
                        &owning_cache.structures.storage
                    {
                        let mut out_target = MoveTarget {
                            pos: source.pos(),
                            range: 1,
                        };
                        let mut in_target = MoveTarget {
                            pos: storage.pos(),
                            range: 1,
                        };

                        let out_steps = out_target
                            .find_path_to(
                                storage.pos(),
                                memory,
                                MoveOptions::default().path_age(u8::MAX),
                            )
                            .len() as u128;
                        let in_steps = in_target
                            .find_path_to(
                                source.pos(),
                                memory,
                                MoveOptions::default().path_age(u8::MAX),
                            )
                            .len() as u128;

                        (out_steps, in_steps)
                    } else {
                        let spawn = owning_cache
                            .spawn_center
                            .unwrap()
                            .as_position(&owning_cache.room_name);

                        let mut out_target = MoveTarget {
                            pos: source.pos(),
                            range: 1,
                        };
                        let mut in_target = MoveTarget {
                            pos: spawn,
                            range: 1,
                        };

                        let out_steps = out_target
                            .find_path_to(spawn, memory, MoveOptions::default().path_age(u8::MAX))
                            .len() as u128;
                        let in_steps = in_target
                            .find_path_to(
                                source.pos(),
                                memory,
                                MoveOptions::default().path_age(u8::MAX),
                            )
                            .len() as u128;

                        (out_steps, in_steps)
                    };

                    carry_requirement += source_ept * (out_steps + in_steps);
                }
            }
        }

        for source in &owning_cache.resources.sources {
            let source_ept = (source.calculate_work_parts() * 2) as u128;
            let source = source.source.clone();

            let (out_steps, in_steps) = if let Some(storage) = &owning_cache.structures.storage {
                let mut out_target = MoveTarget {
                    pos: source.pos(),
                    range: 1,
                };
                let mut in_target = MoveTarget {
                    pos: storage.pos(),
                    range: 1,
                };

                let out_steps = out_target
                    .find_path_to(
                        storage.pos(),
                        memory,
                        MoveOptions::default().path_age(u8::MAX),
                    )
                    .len() as u128;
                let in_steps = in_target
                    .find_path_to(
                        source.pos(),
                        memory,
                        MoveOptions::default().path_age(u8::MAX),
                    )
                    .len() as u128;

                (out_steps, in_steps)
            } else {
                let spawn = owning_cache
                    .spawn_center
                    .unwrap()
                    .as_position(&owning_cache.room_name);

                let mut out_target = MoveTarget {
                    pos: source.pos(),
                    range: 1,
                };
                let mut in_target = MoveTarget {
                    pos: spawn,
                    range: 1,
                };

                let out_steps = out_target
                    .find_path_to(spawn, memory, MoveOptions::default().path_age(u8::MAX))
                    .len() as u128;
                let in_steps = in_target
                    .find_path_to(
                        source.pos(),
                        memory,
                        MoveOptions::default().path_age(u8::MAX),
                    )
                    .len() as u128;

                (out_steps, in_steps)
            };

            carry_requirement += source_ept * (out_steps + in_steps);
        }

        let wanted_hauler_count = (carry_requirement as f32) / (carry_count as f32 * 50.0);

        let mut hauler_count = if wanted_hauler_count < 3.0 {
            3
        } else {
            wanted_hauler_count.round() as u32
        };

        //if wanted_hauler_count > (f32::max(2.0, 15.0 / owning_cache.structures.controller.as_ref().unwrap().controller.level() as f32) * harvester_count as f32).round() {
        //    hauler_count = (f32::max(2.0, 15.0 / owning_cache.structures.controller.as_ref().unwrap().controller.level() as f32) * harvester_count as f32).round() as u32;
        //}

        let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

        let max = match owning_cache.rcl {
            1 => 20,
            2 => 40,
            3 => 70,
            4 => 100,
            5 => 100,
            6 => 100,
            7 => 100,
            8 => 100,
            _ => 100,
        };

        let clamped = hauler_count.clamp(3, max);
        room_memory.hauler_count = clamped;

        info!("[HAULER SCAN] Initiated hauler scan for room {} - hauler count: {} - carry requirement: {}", room.name(), clamped, carry_requirement);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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

            if current_count_for_role <= (*required_count_for_role).try_into().unwrap() && required_count_for_role > &0 {
                let spawn_request = match required_role {
                    Role::Harvester => harvester(&room, room_cache, &mut cache.spawning),
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

                if let Some(spawn_request) = spawn_request {
                    let can_spawn = cache.spawning.can_room_spawn_creep(&room, room_cache, &spawn_request);

                    if can_spawn {
                        let spawned = cache.spawning.room_spawn_creep(&room, memory, room_cache, &spawn_request);

                        if spawned {
                            spawned_this_tick = true;
                        }
                    } else {
                        break;
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

                        continue;
                    }
                }
            }
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn randomize_top_priorities(requests: Vec<SpawnRequest>) -> Vec<SpawnRequest> {
    let mut top_scorers = Vec::new();
    if requests.is_empty() {
        return top_scorers;
    }

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