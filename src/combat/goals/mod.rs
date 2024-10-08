use log::info;
use screeps::{game, Creep};

use crate::{constants, memory::ScreepsMemory, room::cache::RoomCache};

pub mod room_reservation;
pub mod remote_defense;
pub mod remote_invader_cleanup;
pub mod room_claim;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_goal_handlers(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let pre_goals = game::cpu::get_used();

    room_reservation::run_goal(memory, cache);
    remote_defense::run_goal(memory, cache);
    remote_invader_cleanup::run_goal(memory, cache);
    room_claim::run_goal(memory, cache);

    let post_goals = game::cpu::get_used();
    memory.stats.cpu.goal_execution = post_goals - pre_goals;
    info!("[GOALS] Goal handling took {:.2} CPU", post_goals - pre_goals);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn determine_group_attack_power(creeps: &Vec<Creep>) -> u32 {
    let mut total_power = 0;

    for creep in creeps {
        let body = creep.body();
        let attack_power = body.iter().map(|p | constants::part_attack_weight(&p.part())).sum::<u32>();

        total_power += attack_power;
    }

    total_power
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn determine_single_attack_power(creep: &Creep) -> u32 {
    let body = creep.body();
    body.iter().map(|p| constants::part_attack_weight(&p.part())).sum::<u32>()
}