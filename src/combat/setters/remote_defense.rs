use screeps::{game, Part, SharedCreepProperties};

use crate::{allies, combat::goals::determine_single_attack_power, config, constants::HOSTILE_PARTS, goal_memory::{AttackingCreep, RemoteDefenseGoal}, memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn determine_remote_defense_needs(cache: &mut RoomCache, memory: &mut ScreepsMemory) {
    let mark_dangerous = Vec::new();

    for remote_name in memory.remote_rooms.keys() {
        if let Some(remote_cache) = cache.rooms.get_mut(remote_name) {

            // The handler will take care of value refreshing, so we can skip if we already have a goal
            if memory.goals.remote_defense.contains_key(remote_name) {
                continue;
            }

            let hostile_creeps = &remote_cache.creeps.enemy_creeps.iter().filter(|c| c.body().iter().any(|p| HOSTILE_PARTS.contains(&p.part())) && !allies::is_ally(&c.owner().username(), Some(*remote_name))).collect::<Vec<_>>();

            if hostile_creeps.is_empty() {
                continue;
            }

            let mut goal = RemoteDefenseGoal {
                defending_remote: *remote_name,
                power_rescan_tick: 0,
                total_attack_power: 0,
                attacker_names: Vec::new(),
                attacking_creeps: Vec::new(),
                creeps_assigned: Vec::new(),

                invaders: false,
            };

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

            goal.invaders = invader_only_attack(&goal);
            goal.power_rescan_tick = if goal.invaders { game::time() + 100 } else { game::time() + 10 };

            memory.goals.remote_defense.insert(*remote_name, goal);
        }
    }

    for room_name in mark_dangerous {
        if let Some(remote_memory) = memory.remote_rooms.get_mut(room_name) {
            remote_memory.under_attack = true;
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn invader_only_attack(goal: &RemoteDefenseGoal) -> bool {
    for attacker in &goal.attacker_names {
        if attacker != config::INVADER_USERNAME {
            return false;
        }
    }

    true
}