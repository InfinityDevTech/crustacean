use screeps::{game, Part, RoomName, SharedCreepProperties};

use crate::{goal_memory::RoomReservationGoal, memory::{CreepMemory, Role, ScreepsMemory}, room::cache::tick_cache::RoomCache, utils::{self, get_body_cost, get_unique_id, role_to_name}};

pub fn run_goal(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let cloned_goals = memory.goals.room_reservation.clone();
    let reservation_goals = cloned_goals.keys();

    for goal_room in reservation_goals {
        attain_reservation(goal_room, memory, cache);
    }
}

pub fn clear_creeps(goal: &mut RoomReservationGoal) {
    let mut new_creeps = Vec::new();

    for creep in &goal.creeps_assigned {
        let creep = game::creeps().get(creep.to_string());

        if creep.is_none() {
            continue;
        } else {
            new_creeps.push(creep.unwrap().name().to_string());
        }
    }

    goal.creeps_assigned = new_creeps;
}

pub fn attain_reservation(target_room: &RoomName, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let goal = memory.goals.room_reservation.get_mut(target_room).unwrap();

    clear_creeps(goal);
    let current_parts = get_claim_parts(goal);

    if goal.accessible_reservation_spots == 0 {
        memory.goals.room_reservation.remove(target_room);
        return;
    }

    if current_parts < 2 {
        let new_creep = spawn_creep(goal, cache);
        if let Some(new) = new_creep {
            goal.creeps_assigned.push(new);
        }
    }
}

pub fn get_claim_parts(goal: &mut RoomReservationGoal) -> u8 {
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

pub fn spawn_creep(goal: &RoomReservationGoal, cache: &mut RoomCache) -> Option<String> {
    let room = utils::find_closest_owned_room(&goal.reservation_target, cache);

    if let Some(best_spawned) = room {
        let room = game::rooms().get(best_spawned).unwrap();

        let energy_storage = room.energy_capacity_available();

        let body = if energy_storage > 1300 {
            let mut body = vec![Part::Claim, Part::Move];

            let stamp_cost = 650;
            let mut current_cost = 650;

            while current_cost < energy_storage {
                body.push(Part::Claim);
                body.push(Part::Move);

                current_cost += 650;
            }

            body
        } else {
            vec![Part::Claim, Part::Move]
        };
        let cost = get_body_cost(&body);

        let mut creep_memory = CreepMemory {
            role: Role::Reserver,
            owning_room: best_spawned,
            target_room: Some(goal.reservation_target),
            ..Default::default()
        };

        let name = format!("{}-{}-{}", role_to_name(Role::Reserver), best_spawned, get_unique_id());

        let req = cache.spawning.create_room_spawn_request(Role::Reserver, body, 4.0, cost, best_spawned, Some(creep_memory), None, Some(name.clone()));
        if let Some(reqs) = cache.spawning.room_spawn_queue.get_mut(&best_spawned) {
            reqs.push(req);
        } else {
            cache.spawning.room_spawn_queue.insert(best_spawned, vec![req]);
        }

        Some(name.clone());
    }

    None
}