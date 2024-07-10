use log::info;
use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{find, game, Color, Creep, HasPosition, SharedCreepProperties, StructureProperties, StructureType};

use crate::{
    config, memory::{Role, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_bulldozer(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let room_cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    if creep.spawning() {
        return;
    }

    let mut owned = room_cache.creeps.owned_creeps.clone();
    let mut nearby_creeps = owned.values().filter(|c| c.pos().get_range_to(creep.pos()) <= 3).collect::<Vec<&Creep>>();
    // sort by lowest to highest hits
    nearby_creeps.sort_by_key(|c| c.hits());

    let health_percent = if nearby_creeps.first().is_some() {
        let nearby_creep = nearby_creeps.first().unwrap();
        let health_percent = nearby_creep.hits() as f32 / nearby_creep.hits_max() as f32 * 100.0;
        health_percent
    } else {
        100.0
    };

    let my_health_percent = creep.hits() as f32 / creep.hits_max() as f32 * 100.0;

    info!("{}: Health: {}% My Health: {}%", creep.name(), health_percent, my_health_percent);

    if (health_percent < 100.0 || my_health_percent < 100.0) && my_health_percent > health_percent {
        if let Some(creep) = nearby_creeps.first() {
            if creep.pos().is_near_to(creep.pos()) {
                creep.heal(*creep);
            } else {
                creep.ranged_heal(*creep);
                creep.better_move_to(memory, room_cache, creep.pos(), 1, MoveOptions::default().avoid_enemies(true).path_age(1));
            }
        }
    } else if my_health_percent < 100.0 {
        creep.heal(creep);
    }

    let creep_memory = memory.creeps.get_mut(&creep.name());
    if creep_memory.is_none() || creep.spawning() {
        return;
    }
    let creep_memory = creep_memory.unwrap();

    if let Some(flag) = game::flags().get("bulldozeRoom".to_string()) {
        if creep.room().unwrap().name() == flag.pos().room_name() {

            if flag.color() == Color::Blue {
                if creep.pos().is_near_to(flag.pos()) {
                    creep.bsay("👁️", true);
                } else {
                    creep.better_move_to(memory, room_cache, flag.pos(), 1, MoveOptions::default().avoid_enemies(true).path_age(1));
                }
                return;
            }

            if flag.color() == Color::Green {
                if creep.pos().is_near_to(flag.pos()) {
                    creep.bsay("JK - <3 U", true);
                } else {
                    creep.bsay("DIE DIE DIE", true);
                    creep.better_move_to(memory, room_cache, flag.pos(), 1, MoveOptions::default().path_age(1));
                }
                return;
            }

            let mut rng = StdRng::seed_from_u64(game::time() as u64);
            let to_say = config::ATTACK_SIGNS[rng.gen_range(0..config::ATTACK_SIGNS.len())];
            creep.bsay(to_say, true);

            let enemies = creep.pos().find_closest_by_path(find::HOSTILE_CREEPS, None);
            if let Some(enemy) = enemies {
                if creep.attack(&enemy) == Err(screeps::ErrorCode::NotInRange) {
                    creep.better_move_to(memory, room_cache, enemy.pos(), 1, MoveOptions::default().path_age(1));
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
                        creep.better_move_to(memory, room_cache, structure.pos(), 1, MoveOptions::default().path_age(1));
                    }
                } else {
                    let mut structures = creep.room().unwrap().find(find::STRUCTURES, None);
                    structures.retain(| structure | structure.structure_type() != StructureType::Controller);
                    structures.sort_by_key(|structure| structure.pos().get_range_to(creep.pos()));

                    let structure = structures.first();
                    if let Some(structure) = structure {
                        if creep.pos().is_near_to(structure.pos()) {
                            if let Some(attackabke) = structure.as_attackable() {
                                let _ = creep.attack(attackabke);
                            }
                        } else {
                            creep.better_move_to(memory, room_cache, structure.pos(), 1, MoveOptions::default().path_age(1));
                        }
                    } else {
                        creep.bsay("🚚", false);
                        creep.better_move_to(memory, room_cache, flag.pos(), 2, MoveOptions::default().avoid_enemies(true));
                        //creep_memory.role = Role::Recycler;
                    }
                }
            }
        } else {
            creep.bsay("🚚", false);

            if creep.ticks_to_live() < Some(100) {
                creep_memory.role = Role::Recycler;
            }

            creep.better_move_to(memory, room_cache, flag.pos(), 2, MoveOptions::default().avoid_hostile_rooms(true).avoid_enemies(true));
        }
    } else {
        creep.bsay("❓", false);

        creep_memory.role = Role::Recycler;
    }
}