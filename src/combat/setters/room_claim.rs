use screeps::{game, HasPosition, OwnedStructureProperties};

use crate::{goal_memory::RoomClaimGoal, memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::intents_tracking::FlagExtensionsTracking};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn determine_room_claim_needs(memory: &mut ScreepsMemory, _cache: &mut RoomCache) {
    for flag in game::flags().values() {
        if flag.name().starts_with("claim") {
            let room_name = flag.pos().room_name();

            if let Some(game_room) = game::rooms().get(room_name) {
                if game_room.controller().is_some() && game_room.controller().unwrap().owner().is_some() && game_room.controller().unwrap().level() > 1 {
                    game::flags().get(flag.name()).unwrap().ITremove();
                    continue;
                }
            }

            if memory.goals.room_claim.contains_key(&room_name) {
                continue;
            }

            let goal = RoomClaimGoal {
                claim_target: room_name,
                creeps_assigned: Vec::new(),
            };

            memory.goals.room_claim.insert(room_name, goal);
        }
    }
}