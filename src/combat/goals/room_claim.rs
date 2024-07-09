use std::vec;

use screeps::{game, memory, OwnedStructureProperties, Part, RoomName, SharedCreepProperties};

use crate::{goal_memory::RoomClaimGoal, memory::{CreepMemory, Role, ScreepsMemory}, room::cache::tick_cache::RoomCache, traits::room::RoomExtensions, utils::{self, role_to_name}};

pub fn run_goal(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let cloned_goals = memory.goals.room_claim.clone();
    let invader_goals = cloned_goals.keys();

    for goal_room in invader_goals {
        achieve_goal(goal_room, memory, cache);
    }
}

pub fn clear_creeps(goal: &mut RoomClaimGoal) {
    let mut new_creeps = Vec::new();

    for creep in &goal.creeps_assigned {
        let creep = game::creeps().get(creep.to_string());

        if let Some(creep) = creep {
            new_creeps.push(creep.name());
        }
    }

    goal.creeps_assigned = new_creeps;
}

fn achieve_goal(goal_room: &RoomName, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let goal = memory.goals.room_claim.get_mut(goal_room).unwrap();
    let goal_game_room = game::rooms().get(*goal_room);

    clear_creeps(goal);

    let claimed = if goal_game_room.is_none() {
        false
    } else {
        let controller = goal_game_room.unwrap().controller().unwrap();
        controller.owner().is_some() && controller.owner().unwrap().username() == utils::get_my_username()
    };

    let responsible_room = utils::find_closest_owned_room(goal_room, cache, Some(5));
    if responsible_room.is_none() {
        return;
    }

    // Spawn the claimer
    if !claimed && goal.creeps_assigned.len() == 0 {
        let responsible_cache = cache.rooms.get_mut(&responsible_room.unwrap()).unwrap();

        let claimer_body = vec![Part::Claim, Part::Move];
        let claimer_cost = utils::get_body_cost(&claimer_body);

        let creep_memory = CreepMemory {
            role: Role::Claimer,
            owning_room: responsible_room.unwrap(),
            target_room: Some(*goal_room),
            ..Default::default()
        };

        let name = format!("{}-{}-{}", role_to_name(Role::Claimer), responsible_room.unwrap(), utils::get_unique_id());

        goal.creeps_assigned.push(name.clone());

        let spawn_request = cache.spawning.create_room_spawn_request(Role::Claimer, claimer_body, 4.0, claimer_cost, responsible_room.unwrap(), Some(creep_memory), None, Some(name));

        if let Some(reqs) = cache.spawning.room_spawn_queue.get_mut(&responsible_room.unwrap()) {
            reqs.push(spawn_request);
        } else {
            cache
                .spawning
                .room_spawn_queue
                .insert(responsible_room.unwrap(), vec![spawn_request]);
        }
    }
}

fn get_creep_body() -> Vec<Part> {
    let mut body = Vec::new();

    for _ in 0..10 {
        body.push(Part::Work);
    }

    for _ in 0..20 {
        body.push(Part::Carry);
    }

    for _ in 0..20 {
        body.push(Part::Move);
    }

    body
}