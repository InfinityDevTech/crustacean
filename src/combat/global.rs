use crate::{memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

use super::setters;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_global_goal_setters(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    setters::remote_reservation::determine_reservations(memory, cache);
    setters::remote_invader_cleanup::determine_cleanup(memory, cache);
    setters::remote_defense::determine_remote_defense_needs(cache, memory);
    setters::room_claim::determine_room_claim_needs(memory, cache);
}