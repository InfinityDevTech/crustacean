use screeps::Creep;

use crate::{constants, memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

pub mod room_reservation;
pub mod remote_defense;
pub mod remote_invader_cleanup;
pub mod room_claim;

pub fn run_goal_handlers(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    room_reservation::run_goal(memory, cache);
    remote_defense::run_goal(memory, cache);
    remote_invader_cleanup::run_goal(memory, cache);
    room_claim::run_goal(memory, cache);
}

pub fn determine_group_attack_power(creeps: &Vec<&Creep>) -> u32 {
    let mut total_power = 0;

    for creep in creeps {
        let body = creep.body();
        let attack_power = body.iter().map(|p | constants::part_attack_weight(&p.part())).sum::<u32>();

        total_power += attack_power;
    }

    total_power
}

pub fn determine_single_attack_power(creep: &Creep) -> u32 {
    let body = creep.body();
    body.iter().map(|p| constants::part_attack_weight(&p.part())).sum::<u32>()
}