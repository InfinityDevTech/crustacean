use log::info;
use screeps::{find, game, Creep, HasPosition, SharedCreepProperties};

use crate::{
    config,
    memory::{EnemyPlayer, Role, ScreepsMemory},
    room::cache::heap_cache::HealthChangeType, utils::name_to_role,
};

pub fn decay_hate(memory: &mut ScreepsMemory) {
    for enemy in memory.enemy_players.values_mut() {
        if enemy.last_attack - game::time() <= config::TICKS_BEFORE_DECAY {
            enemy.decrement_hate(config::HATE_DECAY_RATE);
        }
    }
}

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

        let offending_user = if memory.enemy_players.contains_key(&offending_user) {
            memory.enemy_players.get_mut(&offending_user).unwrap()
        } else {
            let enemy = EnemyPlayer {
                username: offending_user.clone(),
                hate: 0.0,
                owned_rooms: vec![],
                reserved_rooms: vec![],
                last_attack: 0,
            };

            memory.enemy_players.insert(offending_user.clone(), enemy);
            memory.enemy_players.get_mut(&offending_user).unwrap()
        };

        info!("{} has been attacked by {}", offending_user.username, creep.name());

        if health_type == HealthChangeType::Damage {
            offending_user.increment_hate(config::HATE_CREEP_ATTACK_WEIGHT);
        } else if health_type == HealthChangeType::Heal {
            offending_user.decrement_hate(config::HATE_CREEP_HEAL_WEIGHT)
        }

        offending_user.last_attack = game::time();
    }
}
