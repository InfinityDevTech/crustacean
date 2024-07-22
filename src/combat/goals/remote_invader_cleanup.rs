use log::info;
use screeps::{game, Part, ResourceType, RoomName, SharedCreepProperties, StructureProperties};

use crate::{
    goal_memory::RemoteInvaderCleanup,
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::tick_cache::RoomCache,
    utils,
};
// TODO: Something is telling me that there might be invaders in the room
// at the same time, plan for that please.
// (Potentially: Switch it to a remote_defense goal)
pub fn run_goal(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let cloned_goals = memory.goals.remote_invader_cleanup.clone();

    for (goal_room, goal_mem) in cloned_goals {
        achieve_goal(&goal_room, memory, cache);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn clear_creeps(goal: &mut RemoteInvaderCleanup) {
    let mut new_creeps = Vec::new();

    for creep in &goal.creeps_assigned {
        let gcreep = game::creeps().get(creep.to_string());

        if let Some(gcreep) = gcreep {
            new_creeps.push(gcreep.name());
        } else {
            info!("Creep {} not found", creep);
        }
    }

    goal.creeps_assigned = new_creeps;
}

pub fn achieve_goal(target_room: &RoomName, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let goal = memory
        .goals
        .remote_invader_cleanup
        .get_mut(target_room)
        .unwrap();

        info!("Pre clear {:?}", goal.creeps_assigned);
    clear_creeps(goal);
    info!("Post clear {:?}", goal.creeps_assigned);

    let responsible_room = utils::find_closest_owned_room(target_room, cache, Some(2));

    if let Some(room_cache) = cache.rooms.get_mut(target_room) {
        let invader_core = &room_cache.structures.invader_core;

        if invader_core.is_none() && (room_cache.current_holder != Some("Invader".to_string()) || room_cache.current_holder.is_none()) {
            memory.goals.remote_invader_cleanup.remove(target_room);
            return;
        }

        if invader_core.is_none() {
            info!("No invader core found in room {}", target_room);
            goal.destroyed_core = true;
            return;
        }
    }

    if goal.creeps_assigned.is_empty() && !goal.destroyed_core {
        if let Some(responsible_room) = responsible_room {
            let mut reservation = 0.0;

            if let Some(room_cache) = cache.rooms.get(target_room) {
                reservation = room_cache.reservation as f32;

                if let Some(storage) = room_cache.structures.storage.as_ref() {
                    if storage.store().get_used_capacity(Some(ResourceType::Energy)) < 10000 {
                        return;
                    }
                }
            }

            let stamp = vec![Part::Attack, Part::Move];
            let stamp_cost = utils::get_body_cost(&stamp);

            let mut body = Vec::new();
            let mut current_cost = 0;
            let mut current_attack = 0;

            let responsible_room = cache.rooms.get_mut(&responsible_room).unwrap();
            let available_energy = game::rooms()
                .get(responsible_room.room_name)
                .unwrap()
                .energy_available();

            while current_cost < available_energy {
                if current_cost + available_energy < stamp_cost || current_attack + 1 > 10 {
                    break;
                }

                body.extend(stamp.clone());
                current_cost += stamp_cost;
                current_attack += 1;
            }

            let memory = CreepMemory {
                role: Role::InvaderCoreCleaner,
                owning_room: responsible_room.room_name,
                target_room: Some(*target_room),
                ..CreepMemory::default()
            };

            let name = format!(
                "{}-{}-{}",
                utils::role_to_name(Role::InvaderCoreCleaner),
                responsible_room.room_name,
                utils::get_unique_id()
            );

            // These guys dont fight back, just suck my eco dry is all.
            let mut priority = 5.0;

            priority += reservation as f64 / 100.0;

            let req = cache.spawning.create_room_spawn_request(
                Role::InvaderCoreCleaner,
                body,
                priority,
                current_cost,
                responsible_room.room_name,
                Some(memory),
                None,
                Some(name.clone()),
            );

            goal.creeps_assigned.push(name.clone());

            info!("Spawning invader core cleaner in {} - {}", responsible_room.room_name, name.clone());

            if let Some(reqs) = cache
                .spawning
                .room_spawn_queue
                .get_mut(&responsible_room.room_name)
            {
                reqs.push(req);
            } else {
                cache
                    .spawning
                    .room_spawn_queue
                    .insert(responsible_room.room_name, vec![req]);
            }
        } else {
            info!("No responsible room found for {}", target_room);
        }
    }
}
