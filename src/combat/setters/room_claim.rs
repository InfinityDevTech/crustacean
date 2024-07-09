use screeps::{game, HasPosition};

use crate::{goal_memory::RoomClaimGoal, memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

pub fn determine_room_claim_needs(memory: &mut ScreepsMemory, _cache: &mut RoomCache) {
    for flag in game::flags().values() {
        if flag.name().starts_with("claim") {
            let room_name = flag.pos().room_name();

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