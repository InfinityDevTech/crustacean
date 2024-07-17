use log::info;
use screeps::{game, Part, RoomName, SharedCreepProperties};

use crate::{
    goal_memory::RoomReservationGoal,
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::tick_cache::RoomCache,
    utils::{self, get_body_cost, get_unique_id, role_to_name},
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_goal(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let cloned_goals = memory.goals.room_reservation.clone();
    let reservation_goals = cloned_goals.keys();

    for goal_room in reservation_goals {
        attain_reservation(goal_room, memory, cache);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn clear_creeps(goal: &mut RoomReservationGoal) {
    let mut new_creeps = Vec::new();

    for creep in &goal.creeps_assigned {
        let creep = game::creeps().get(creep.to_string());

        if let Some(creep) = creep {
            new_creeps.push(creep.name());
        }
    }

    goal.creeps_assigned = new_creeps;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn attain_reservation(
    target_room: &RoomName,
    memory: &mut ScreepsMemory,
    cache: &mut RoomCache,
) {
    let goal = memory.goals.room_reservation.get_mut(target_room).unwrap();

    clear_creeps(goal);
    let current_parts = get_claim_parts(goal);

    if goal.accessible_reservation_spots == 0 {
        memory.goals.room_reservation.remove(target_room);
        return;
    }

    if current_parts < 2 && goal.accessible_reservation_spots > 1 {
        let new_creep = spawn_creep(goal, cache);
        if let Some(new) = new_creep {
            goal.creeps_assigned.push(new);
        }
    }

    if let Some(room) = game::rooms().get(*target_room) {
        let reservation_status = room.controller().unwrap().reservation();
        if let Some(reservation) = reservation_status {
            // Basically, we completed the goal. Soooo, we can remove it
            if reservation.username() == utils::get_my_username()
                && reservation.ticks_to_end() > 1200
            {
                info!("[{}] Successfully reserved remote to satisfactory levels. Removing goal", target_room);
                memory.goals.room_reservation.remove(target_room);
            }
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_claim_parts(goal: &RoomReservationGoal) -> u8 {
    let mut count = 0;

    for creep in &goal.creeps_assigned {
        let creep = game::creeps().get(creep.to_string()).unwrap();

        for part in creep.body() {
            if part.part() == Part::Claim {
                count += 1;
            }
        }
    }

    count
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn spawn_creep(goal: &RoomReservationGoal, cache: &mut RoomCache) -> Option<String> {
    let room = utils::find_closest_owned_room(&goal.reservation_target, cache, Some(4));

    if let Some(best_spawned) = room {
        let room = game::rooms().get(best_spawned).unwrap();

        // Only at RCL 4 do we really start to care about reserving rooms
        if room.controller().unwrap().level() < 4 {
            return None;
        }

        let energy_storage = room.energy_capacity_available();
        let mut current_claim = 0;

        let body = if energy_storage > 1300 {
            let mut body = vec![Part::Claim, Part::Move];

            let stamp_cost = 650;
            let mut current_cost = 650;

            while current_cost < energy_storage {
                if current_cost + stamp_cost > energy_storage || current_claim >= 3 {
                    break;
                }

                body.push(Part::Claim);
                body.push(Part::Move);

                current_claim += 1;
                current_cost += 650;
            }

            body
        } else {
            vec![Part::Claim, Part::Move]
        };
        let cost = get_body_cost(&body);

        // If we can only make one part, and we cant have 2 creeps, then we dont spawn
        if goal.accessible_reservation_spots == 1 && current_claim == 1 {
            return None;
        }

        let creep_memory = CreepMemory {
            role: Role::Reserver,
            owning_room: best_spawned,
            target_room: Some(goal.reservation_target),
            ..Default::default()
        };

        let name = format!(
            "{}-{}-{}",
            role_to_name(Role::Reserver),
            best_spawned,
            get_unique_id()
        );

        let mut priority = 4.0;

        // We have two spawns, eco creeps can cope.
        if room.controller().unwrap().level() >= 7 {
            priority *= 2.0;
        }

        // if we have one claim part, its doing nothing.
        // So we can bump the priority to assist the 1 part creep
        if get_claim_parts(goal) == 1 {
            priority *= 1.5;
        }

        let req = cache.spawning.create_room_spawn_request(
            Role::Reserver,
            body,
            priority,
            cost,
            best_spawned,
            Some(creep_memory),
            None,
            Some(name.clone()),
        );
        if let Some(reqs) = cache.spawning.room_spawn_queue.get_mut(&best_spawned) {
            reqs.push(req);
        } else {
            cache
                .spawning
                .room_spawn_queue
                .insert(best_spawned, vec![req]);
        }

        return Some(name)
    }

    None
}
