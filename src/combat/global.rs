use log::info;
use screeps::game;

use crate::{memory::ScreepsMemory, room::cache::RoomCache};

use super::setters;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_global_goal_setters(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let pre_goals = game::cpu::get_used();

    setters::remote_reservation::determine_reservations(memory, cache);
    setters::remote_invader_cleanup::determine_cleanup(memory, cache);
    setters::remote_defense::determine_remote_defense_needs(cache, memory);
    setters::room_claim::determine_room_claim_needs(memory, cache);

    let post_goals = game::cpu::get_used();
    memory.stats.cpu.goal_creation = post_goals - pre_goals;
    info!("[GOALS] Global goal creation took {:.2} CPU", post_goals - pre_goals);
}