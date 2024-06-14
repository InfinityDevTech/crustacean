use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{find, game, Color, Creep, HasPosition, OwnedStructureProperties, SharedCreepProperties, StructureProperties, StructureType};

use crate::{
    config, memory::ScreepsMemory, movement::move_target::MoveOptions, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions
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
                    creep.better_move_to(creep_memory, room_cache, flag.pos(), 1, MoveOptions::default().avoid_enemies(true));
                }
                return;
            }

            if flag.color() == Color::Green {
                if creep.pos().is_near_to(flag.pos()) {
                    let _ = creep.say("JK - <3 U", true);
                } else {
                    let _ = creep.say("DIE DIE DIE", true);
                    creep.better_move_to(creep_memory, room_cache, flag.pos(), 1, MoveOptions::default().avoid_enemies(true));
                }
                return;
            }

            let mut rng = StdRng::seed_from_u64(game::time() as u64);
            let to_say = config::ATTACK_SIGNS[rng.gen_range(0..config::ATTACK_SIGNS.len())];
            let _ = creep.say(to_say, true);

            if let Some(controller) = creep.room().unwrap().controller() {
                if controller.my() {
                    let _ = creep.say("🏳️", true);
                    return;
                }

                if creep.pos().is_near_to(controller.pos()) {
                    let _ = creep.attack_controller(&controller);
                } else {
                    creep.better_move_to(creep_memory, room_cache, controller.pos(), 1, MoveOptions::default().avoid_enemies(true));
                }
            }
        } else {
            let _ = creep.say("🚚", false);
            creep.better_move_to(creep_memory, room_cache, flag.pos(), 2, MoveOptions::default().avoid_enemies(true));
        }
    } else {
        let _ = creep.say("❓", false);
    }
}
