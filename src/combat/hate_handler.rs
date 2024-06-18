use std::str::FromStr;

use screeps::{find, game, Creep, HasPosition, ObjectId, Room, SharedCreepProperties};

use crate::{
    config,
    memory::{EnemyPlayer, Role, ScreepsMemory},
    room::cache::{heap_cache::HealthChangeType, tick_cache::RoomCache}, utils::name_to_role,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn increment_hate(memory: &mut ScreepsMemory, hate: f32, player_name: String) {
    if let Some(enemy) = memory.enemy_players.get_mut(&player_name) {
        enemy.increment_hate(hate);
        enemy.last_attack = game::time();
    } else {
        memory.enemy_players.insert(
            player_name.clone(),
            EnemyPlayer {
                username: player_name,
                owned_rooms: vec![],
                reserved_rooms: vec![],
                hate,
                last_attack: game::time(),
            },
        );
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn decay_hate(memory: &mut ScreepsMemory) {
    for enemy in memory.enemy_players.values_mut() {
        if enemy.last_attack - game::time() <= config::TICKS_BEFORE_DECAY {
            enemy.decrement_hate(enemy.hate * config::HATE_DECAY_PERCENTEAGE);
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn process_health_event(creep: &Creep, memory: &mut ScreepsMemory, health_type: HealthChangeType) {
    let offending_creeps = creep.pos().find_in_range(find::HOSTILE_CREEPS, 3);

    let creep_role = name_to_role(&creep.name());

    // This is done for a reason, if a scout enters an enemy room
    // and its a new player with primitive defense that just finds enemy creeps
    // and attacks, we don't want to increment hate for that player
    // (Just to be nice 😁)
    if creep_role.is_none() || creep_role == Some(Role::Scout) {
        return;
    }

    if !offending_creeps.is_empty() {
        let offending_user = offending_creeps.first().unwrap().owner().username();

        if offending_user == config::USERNAME {
            return;
        }

        let weight = if health_type == HealthChangeType::Heal {
            config::HATE_CREEP_HEAL_WEIGHT
        } else {
            config::HATE_CREEP_ATTACK_WEIGHT
        };

        increment_hate(memory, weight, offending_user.to_string());
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn process_room_event_log(room: &Room, memory: &mut ScreepsMemory, _cache: &mut RoomCache) {
    let event_log = room.get_event_log();
    for event in event_log {
        match event.event {
            screeps::EventType::Attack(attack_event) => {
                let attacker = event.object_id;
                let attackee = attack_event.target_id;

                if game::get_object_by_id_typed::<Creep>(&ObjectId::from_str(&attackee).unwrap()).is_none() {
                    let attacker: Option<Creep> = game::get_object_by_id_typed(&ObjectId::from_str(&attacker).unwrap());
                    if let Some(attacker) = attacker  {
                        let owner = attacker.owner().username();

                        if owner == config::USERNAME {
                            continue;
                        }

                        increment_hate(memory, config::HATE_CREEP_KILLED_WEIGHT, owner.to_string());
                    }
                } else {
                    let attacker: Option<Creep> = game::get_object_by_id_typed(&ObjectId::from_str(&attacker).unwrap());
                    if let Some(attacker) = attacker  {
                        let owner = attacker.owner().username();

                        if owner == config::USERNAME {
                            continue;
                        }

                        increment_hate(memory, config::HATE_CREEP_ATTACK_WEIGHT, owner.to_string());
                    }
                }
            },
            screeps::EventType::AttackController => {},
            _ => {}
        }
    }
}
