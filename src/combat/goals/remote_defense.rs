use log::info;
use screeps::{game, Creep, Part, Room, RoomName, SharedCreepProperties};

use crate::{constants::{self, part_attack_weight, HOSTILE_PARTS}, goal_memory::{AttackingCreep, RemoteDefenseGoal}, memory::{CreepMemory, Role, ScreepsMemory}, room::cache::tick_cache::RoomCache, utils::{self, get_body_cost, get_unique_id, role_to_name}};

use super::{determine_group_attack_power, determine_single_attack_power};

pub fn run_goal(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let cloned_goals = memory.goals.remote_defense.clone();
    let defense_goals = cloned_goals.keys();

    for goal_room in defense_goals {
        attain_goal(goal_room, memory, cache);
    }
}

pub fn clear_creeps(goal: &mut RemoteDefenseGoal) {
    let mut new_creeps = Vec::new();

    for creep in &goal.creeps_assigned {
        let creep = game::creeps().get(creep.to_string());

        if let Some(creep) = creep {
            new_creeps.push(creep.name());
        }
    }

    goal.creeps_assigned = new_creeps;
}

fn attain_goal(goal_room: &RoomName, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if let Some(remote_mem) = memory.remote_rooms.get_mut(goal_room) {
        remote_mem.under_attack = true;
    } else {
        memory.goals.remote_defense.remove(goal_room);

        return;
    }

    let goal = memory.goals.remote_defense.get_mut(goal_room).unwrap();

    let closest_room = if goal.invaders {
        utils::find_closest_owned_room(&goal.defending_remote, cache, Some(4))
    } else {
        utils::find_closest_owned_room(&goal.defending_remote, cache, Some(6))
    };

    clear_creeps(goal);

    if let Some(room) = closest_room {
        if game::time() == goal.power_rescan_tick {
            if let Some(remote_cache) = cache.rooms.get_mut(&goal.defending_remote) {
                let hostile_creeps = &remote_cache.creeps.enemy_creeps.iter().filter(|c| c.body().iter().any(|p| HOSTILE_PARTS.contains(&p.part()))).collect::<Vec<_>>();

                goal.attacking_creeps.clear();
                goal.attacker_names.clear();

                if hostile_creeps.is_empty() {
                    memory.goals.remote_defense.remove(goal_room);

                    if let Some(remote_mem) = memory.rooms.get_mut(goal_room) {
                        remote_mem.under_attack = false;
                    }

                    return;
                }

                for creep in hostile_creeps {
                    let attack_power = determine_single_attack_power(creep);
                    let body = creep.body().iter().map(|part| part.part()).collect::<Vec<Part>>();

                    let attacking_creep = AttackingCreep {
                        creep_name: creep.name(),
                        owner_name: creep.owner().username(),
                        attack_power,
                        body,
                        ttl: creep.ticks_to_live().unwrap_or(0),
                    };

                    goal.total_attack_power += attack_power;

                    if !goal.attacker_names.contains(&attacking_creep.owner_name) {
                        goal.attacker_names.push(attacking_creep.owner_name.clone());
                    }

                    goal.attacking_creeps.push(attacking_creep);
                }
            }

            goal.power_rescan_tick = if goal.invaders { game::time() + 100 } else { game::time() + 10 };
        }

        if let Some(remote_cache) = cache.rooms.get_mut(&goal.defending_remote) {
            let hostile_creeps = &remote_cache.creeps.enemy_creeps_with_attack;

            if hostile_creeps.is_empty() {
                memory.goals.remote_defense.remove(goal_room);

                if let Some(remote_mem) = memory.rooms.get_mut(goal_room) {
                    remote_mem.under_attack = false;
                }

                return;
            }
        }

        let room_cache = cache.rooms.get_mut(&room).unwrap();

        let my_creeps = &goal.creeps_assigned;
        let my_creep_power = determine_group_attack_power(&my_creeps.iter().map(|c| room_cache.creeps.owned_creeps.get(c).unwrap()).collect());

        if my_creep_power < goal.total_attack_power {
            determine_spawn_needs(&game::rooms().get(room).unwrap(), goal, cache);
        }
    }
}

fn determine_spawn_needs(responsible_room: &Room, goal: &mut RemoteDefenseGoal, cache: &mut RoomCache) {
    let stamp = vec![Part::RangedAttack, Part::RangedAttack, Part::Heal, Part::Move, Part::Move, Part::Move];
    let stamp_cost = stamp.iter().map(|part| part.cost()).sum::<u32>();
    let stamp_power = stamp.iter().map(part_attack_weight).sum::<u32>();

    let enemy_power = goal.total_attack_power;

    let mut parts = Vec::new();
    let mut current_cost = 0;
    let mut current_power = 0;

    let energy_available = responsible_room.energy_available();

    while current_cost < energy_available {
        if current_cost + stamp_cost > energy_available || current_power + stamp_power > enemy_power {
            break;
        }

        parts.extend_from_slice(&stamp);
        current_cost += stamp_cost;
        current_power += stamp_power;
    }

    if !parts.is_empty() {
        let creep_name = format!("{}-{}-{}", role_to_name(Role::RemoteDefender), responsible_room.name(), get_unique_id());
        let cost = get_body_cost(&parts);

        let creep_memory = CreepMemory {
            role: Role::RemoteDefender,
            owning_room: responsible_room.name(),
            target_room: Some(goal.defending_remote),
            ..Default::default()
        };

        let req = cache.spawning.create_room_spawn_request(Role::RemoteDefender, parts, 4.5, cost, responsible_room.name(), Some(creep_memory), None, Some(creep_name.clone()));

        if let Some(reqs) = cache.spawning.room_spawn_queue.get_mut(&responsible_room.name()) {
            reqs.push(req);
        } else {
            cache
                .spawning
                .room_spawn_queue
                .insert(responsible_room.name(), vec![req]);
        }

        goal.creeps_assigned.push(creep_name);
    }
}