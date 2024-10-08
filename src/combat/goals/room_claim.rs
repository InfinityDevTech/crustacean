use std::vec;

use log::info;
use screeps::{
    find, game, HasPosition, MapTextStyle, MapVisual, OwnedStructureProperties, Part, Position, Room, RoomCoordinate, RoomName, SharedCreepProperties, StructureType
};

use crate::{
    goal_memory::RoomClaimGoal,
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::RoomCache,
    traits::{intents_tracking::RoomExtensionsTracking, position::RoomXYExtensions},
    utils::{self, distance_transform, new_xy, role_to_name, under_storage_gate},
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

    if let Some(scouting_data) = memory.scouted_rooms.get(goal_room) {
        if scouting_data.owner.is_some() || scouting_data.reserved.is_some() {
            memory.goals.room_claim.remove(goal_room);
            memory.expansion = None;
            return;
        }
    }

    clear_creeps(goal);

    if memory.rooms.len() > game::gcl::level() as usize {
        return;
    }

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

    info!("Claimed, {}, assigned {}", claimed, goal.creeps_assigned.len());

    let pos = Position::new(RoomCoordinate::new(5).unwrap(), RoomCoordinate::new(5).unwrap(), *goal_room);
    MapVisual::text(pos, "🚩".to_string(), MapTextStyle::default());

    // Spawn the claimer
    if !claimed && goal.creeps_assigned.is_empty() {
        let claimer_body = vec![
            Part::Claim,
            Part::Move,
            Part::Move,
            Part::Move,
            Part::Move,
            Part::Move,
        ];
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
            40.0,
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
            let claimer_body =
                get_creep_body(&game::rooms().get(responsible_room.unwrap()).unwrap());
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

            let mut priority = if goal.creeps_assigned.len() <= 1 {
                10.0
            } else {
                4.0
            };

            let cr = cache.rooms.get(&responsible_room.unwrap()).unwrap();

            if cr.rcl < cr.max_rcl {
                return;
            }

            if cache.rooms.get(&responsible_room.unwrap()).unwrap().rcl >= 6 {
                priority *= 5.0;
            }

            if !under_storage_gate(cache.rooms.get(&responsible_room.unwrap()).unwrap(), 1.0) {
                priority = f64::MAX - 5.0;
            }

            info!("Spawning expansion dude with prio {:?}", priority);

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
        if !expansion_cache.structures.spawns.is_empty()
            && expansion_game_room.controller().unwrap().level() >= 2
        {
            memory.goals.room_claim.remove(goal_room);

            return;
        }

        let has_spawn_csite_or_spawn = !expansion_cache.structures.spawns.is_empty()
            || expansion_cache
                .structures
                .construction_sites
                .iter()
                .any(|cs| cs.structure_type() == screeps::StructureType::Spawn);

        if !has_spawn_csite_or_spawn && game::cpu::bucket() >= 2500 {
            if game::rooms().get(responsible_room.unwrap()).is_none() {
                return;
            }

            cache.create_if_not_exists(
                &game::rooms().get(responsible_room.unwrap()).unwrap(),
                memory,
                None,
            );
            let cached_room = cache.rooms.get_mut(&responsible_room.unwrap()).unwrap();

            let available_positions = distance_transform(goal_room, None, true, false);
            let mut available_xy = Vec::new();

            let exits = expansion_game_room.find(find::EXIT, None);
            let mut xy_exits = Vec::new();

            for exit in exits {
                xy_exits.push(new_xy(exit.x(), exit.y()));
            }

            for x in 1..49 {
                for y in 1..49 {
                    let xy = new_xy(x, y);

                    let score = available_positions.get(xy);
                    let mut should_continue = false;

                    for exit in &xy_exits {
                        if exit.get_range_to(xy) <= 8 {
                            should_continue = true;
                        }
                    }

                    if score >= 7 && !should_continue {
                        available_xy.push(xy);
                    }
                }
            }

            let cpos = cached_room.structures.controller.as_ref().map(|controller| controller.pos());

            let mut lowest = u32::MAX;
            let mut lowest_pos = None;

            for pos in available_xy {
                let xy = pos.as_position(goal_room);

                let dist = cpos.unwrap().pos().xy().get_range_to(pos);
                let mut source_dist = 0;

                for source in &cached_room.resources.sources {
                    source_dist += source.source.pos().xy().get_range_to(pos);
                }

                let total_dist = dist + source_dist;

                if game::cpu::get_used() >= 475.0 {
                    break;
                }

                if (total_dist as u32) < lowest {
                    lowest = total_dist as u32;
                    lowest_pos = Some(xy);
                }
            }

            if lowest_pos.is_none() {
                return;
            }

            //let flag = lowest_pos.unwrap().create_flag(
            //    Some(&JsString::from_str("PlacedSpawn").unwrap()),
            //    None,
            //    None,
            //);
            let _ = expansion_game_room.ITcreate_construction_site(lowest_pos.unwrap().x().u8(), lowest_pos.unwrap().y().u8(), StructureType::Spawn, None);
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn get_creep_body(room: &Room) -> Vec<Part> {
    let mut body = Vec::new();

    let stamp = vec![Part::Work, Part::Carry, Part::Move, Part::Move];
    let stamp_cost = stamp.iter().map(|part| part.cost()).sum::<u32>();

    let energy_available = room.energy_capacity_available();
    let mut current_cost = 0;

    while current_cost < energy_available {
        if current_cost + stamp_cost > energy_available {
            break;
        }

        body.extend_from_slice(&stamp);
        current_cost += stamp_cost;
    }

    body
}
