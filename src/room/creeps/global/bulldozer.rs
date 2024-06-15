use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{find, game, Color, Creep, HasPosition, SharedCreepProperties, StructureProperties, StructureType};

use crate::{
    config, memory::{Role, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {

    let room_cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    let creep_memory = memory.creeps.get_mut(&creep.name());
    if creep_memory.is_none() {
        return;
    }
    let creep_memory = creep_memory.unwrap();

    if creep.hits() < creep.hits_max() {
        let _ = creep.heal(creep);
    }

    if let Some(flag) = game::flags().get("bulldozeRoom".to_string()) {
        if creep.room().unwrap().name() == flag.pos().room_name() {

            if flag.color() == Color::Blue {
                if creep.pos().is_near_to(flag.pos()) {
                    let _ = creep.say("👁️", true);
                } else {
                    creep.better_move_to(creep_memory, room_cache, flag.pos(), 1, MoveOptions::default().avoid_enemies(true).path_age(3));
                }
                return;
            }

            if flag.color() == Color::Green {
                if creep.pos().is_near_to(flag.pos()) {
                    let _ = creep.say("JK - <3 U", true);
                } else {
                    let _ = creep.say("DIE DIE DIE", true);
                    creep.better_move_to(creep_memory, room_cache, flag.pos(), 1, MoveOptions::default().avoid_enemies(true).path_age(3));
                }
                return;
            }

            let mut rng = StdRng::seed_from_u64(game::time() as u64);
            let to_say = config::ATTACK_SIGNS[rng.gen_range(0..config::ATTACK_SIGNS.len())];
            let _ = creep.say(to_say, true);

            let enemies = creep.pos().find_closest_by_path(find::HOSTILE_CREEPS, None);
            if let Some(enemy) = enemies {
                if creep.attack(&enemy) == Err(screeps::ErrorCode::NotInRange) {
                    creep.better_move_to(creep_memory, room_cache, enemy.pos(), 1, MoveOptions::default().avoid_enemies(true).path_age(3));

                    creep_memory.path = None;
                }
            } else {
                let mut structure = creep.room().unwrap().find(find::HOSTILE_STRUCTURES, None);
                structure.retain(| structure | structure.structure_type() != StructureType::Controller);
                structure.sort_by_key(|structure| structure.pos().get_range_to(creep.pos()));
                let structure = structure.first();
                if let Some(structure) = structure {
                    if creep.pos().is_near_to(structure.pos()) {
                        if let Some(attackabke) = structure.as_attackable() {
                            let _ = creep.attack(attackabke);
                        }
                    } else {
                        creep.better_move_to(creep_memory, room_cache, structure.pos(), 1, MoveOptions::default().avoid_enemies(true).path_age(3));

                        creep_memory.path = None;
                    }
                } else {
                    let _ = creep.say("🚚", false);
                    creep.better_move_to(creep_memory, room_cache, flag.pos(), 2, MoveOptions::default().avoid_enemies(true));
                    //creep_memory.role = Role::Recycler;
                }
            }
        } else {
            let _ = creep.say("🚚", false);

            if creep.ticks_to_live() < Some(100) {
                creep_memory.role = Role::Recycler;
            }

            creep.better_move_to(creep_memory, room_cache, flag.pos(), 2, MoveOptions::default().avoid_enemies(true));
        }
    } else {
        let _ = creep.say("❓", false);

        creep_memory.role = Role::Recycler;
    }
}
