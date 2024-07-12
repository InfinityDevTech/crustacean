use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{game, Color, Creep, HasPosition, OwnedStructureProperties, SharedCreepProperties};

use crate::{
    config, memory::{Role, ScreepsMemory}, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking}, utils::get_my_username
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_unclaimer(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {

    let room_cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    let creep_memory = memory.creeps.get_mut(&creep.name());
    if creep_memory.is_none() || creep.spawning() {
        return;
    }
    let creep_memory = creep_memory.unwrap();

    if creep.ticks_to_live() < Some(100) {
        creep_memory.role = Role::Recycler;
    }

    if creep.hits() < creep.hits_max() {
        let _ = creep.ITheal(creep);
    }

    if let Some(flag) = game::flags().get("bulldozeRoom".to_string()) {
        if creep.room().unwrap().name() == flag.pos().room_name() {

            if flag.color() == Color::Blue {
                if creep.pos().is_near_to(flag.pos()) {
                    creep.bsay("👁️", true);
                } else {
                    creep.better_move_to(memory, room_cache, flag.pos(), 1, MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(true));
                }
                return;
            }

            if flag.color() == Color::Green {
                if creep.pos().is_near_to(flag.pos()) {
                    creep.bsay("JK - <3 U", true);
                } else {
                    creep.bsay("DIE DIE DIE", true);
                    creep.better_move_to(memory, room_cache, flag.pos(), 1, MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(true));
                }
                return;
            }

            let mut rng = StdRng::seed_from_u64(game::time() as u64);
            let to_say = config::ATTACK_SIGNS[rng.gen_range(0..config::ATTACK_SIGNS.len())];
            creep.bsay(to_say, true);

            if let Some(controller) = creep.room().unwrap().controller() {
                if controller.my() {
                    creep.bsay("🏳️", true);
                    return;
                }

                if creep.pos().is_near_to(controller.pos()) {
                    if controller.reservation().is_some() && controller.reservation().unwrap().username() == get_my_username() {
                        let _ = creep.ITreserve_controller(&controller);
                        return;
                    }

                    if controller.reservation().is_none() && memory.remote_rooms.contains_key(&creep.room().unwrap().name()) {
                        let _ = creep.ITreserve_controller(&controller);
                    } else if controller.owner().is_none() && memory.goals.room_claim.contains_key(&creep.room().unwrap().name()) {
                        let _ = creep.ITclaim_controller(&controller);
                    } else {
                        let res = creep.ITattack_controller(&controller);
                        creep.bsay(&format!("{:?}", res), true);
                    }
                } else {
                    creep.bsay("🚚", false);
                    creep.better_move_to(memory, room_cache, controller.pos(), 1, MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(true));
                }
            }
        } else {
            creep.bsay("🚚", false);
            creep.better_move_to(memory, room_cache, flag.pos(), 2, MoveOptions::default().avoid_enemies(true).avoid_hostile_rooms(true));
        }
    } else {
        creep.bsay("❓", false);
    }
}
