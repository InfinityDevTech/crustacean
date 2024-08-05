use std::vec;

use log::info;
use screeps::{find, game, Flag, HasPosition, OwnedStructureProperties, Part, RoomName, SharedCreepProperties, StructureType};

use crate::{
    goal_memory::RoomClaimGoal,
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::RoomCache,
    traits::intents_tracking::RoomExtensionsTracking,
    utils::{self, role_to_name},
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_goal(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let cloned_goals = memory.goals.room_claim.clone();
    let invader_goals = cloned_goals.keys();

    for goal_room in invader_goals {
        achieve_goal(goal_room, memory, cache);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn achieve_goal(goal_room: &RoomName, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let goal = memory.goals.room_claim.get_mut(goal_room).unwrap();
    let goal_game_room = game::rooms().get(*goal_room);

    clear_creeps(goal);

    let claimed = if goal_game_room.is_none() {
        false
    } else {
        let controller = goal_game_room.unwrap().controller().unwrap();
        controller.owner().is_some()
            && controller.owner().unwrap().username() == utils::get_my_username()
    };

    let responsible_room = utils::find_closest_owned_room(goal_room, cache, Some(5));
    if responsible_room.is_none() {
        return;
    }

    // Spawn the claimer
    if !claimed && goal.creeps_assigned.is_empty() {
        let claimer_body = vec![Part::Claim, Part::Move];
        let claimer_cost = utils::get_body_cost(&claimer_body);

        let creep_memory = CreepMemory {
            role: Role::Claimer,
            owning_room: responsible_room.unwrap(),
            target_room: Some(*goal_room),
            ..Default::default()
        };

        let name = format!(
            "{}-{}-{}",
            role_to_name(Role::Claimer),
            responsible_room.unwrap(),
            utils::get_unique_id()
        );

        goal.creeps_assigned.push(name.clone());

        let spawn_request = cache.spawning.create_room_spawn_request(
            Role::Claimer,
            claimer_body,
            4.0,
            claimer_cost,
            responsible_room.unwrap(),
            Some(creep_memory),
            None,
            Some(name),
        );

        if let Some(reqs) = cache
            .spawning
            .room_spawn_queue
            .get_mut(&responsible_room.unwrap())
        {
            reqs.push(spawn_request);
        } else {
            cache
                .spawning
                .room_spawn_queue
                .insert(responsible_room.unwrap(), vec![spawn_request]);
        }
    } else if claimed {
        if goal.creeps_assigned.len() < 3 {

            let claimer_body = get_creep_body();
            let claimer_cost = utils::get_body_cost(&claimer_body);

            let creep_memory = CreepMemory {
                role: Role::ExpansionBuilder,
                owning_room: responsible_room.unwrap(),
                target_room: Some(*goal_room),
                ..Default::default()
            };

            let name = format!(
                "{}-{}-{}",
                role_to_name(Role::ExpansionBuilder),
                responsible_room.unwrap(),
                utils::get_unique_id()
            );

            let priority = if goal.creeps_assigned.len() <= 1 {
                10.0
            } else {
                4.0
            };

            goal.creeps_assigned.push(name.clone());

            let spawn_request = cache.spawning.create_room_spawn_request(
                Role::ExpansionBuilder,
                claimer_body,
                priority,
                claimer_cost,
                responsible_room.unwrap(),
                Some(creep_memory),
                None,
                Some(name),
            );

            if let Some(reqs) = cache
                .spawning
                .room_spawn_queue
                .get_mut(&responsible_room.unwrap())
            {
                reqs.push(spawn_request);
            } else {
                cache
                    .spawning
                    .room_spawn_queue
                    .insert(responsible_room.unwrap(), vec![spawn_request]);
            }
        }

        let expansion_game_room = game::rooms().get(*goal_room).unwrap();
        cache.create_if_not_exists(&expansion_game_room, memory, None);
        let expansion_cache = cache.rooms.get_mut(goal_room);

        if expansion_cache.is_none() {
            return;
        }

        let expansion_cache = expansion_cache.unwrap();

        // If we built the spawn, the room can be removed from the goal list
        // Since it can handle itself now
        // RCL 2 since we get a safemode.
        if !expansion_cache.structures.spawns.is_empty() && expansion_game_room.controller().unwrap().level() >= 2 {
            memory.goals.room_claim.remove(goal_room);
        }

        let has_spawn_csite_or_spawn = !expansion_cache.structures.spawns.is_empty()
            || expansion_cache.structures.construction_sites().iter().any(|cs| {
                cs.structure_type() == screeps::StructureType::Spawn
            });

        if !has_spawn_csite_or_spawn {
            let find = expansion_game_room.find(find::FLAGS, None);
            let flag_pos = find.iter().filter(|f| f.name().starts_with("claim")).collect::<Vec<&Flag>>();

            if flag_pos.is_empty() {
                return;
            }

            let flag = flag_pos[0];
            let res = expansion_game_room.ITcreate_construction_site(flag.pos().x().u8(), flag.pos().y().u8(), StructureType::Spawn, None);
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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
