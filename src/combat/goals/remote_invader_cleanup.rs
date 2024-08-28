use log::info;
use screeps::{game, MapTextStyle, MapVisual, Part, Position, ResourceType, RoomCoordinate, RoomName, SharedCreepProperties};

use crate::{
    goal_memory::RemoteInvaderCleanup,
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::RoomCache,
    utils,
};
// TODO: Something is telling me that there might be invaders in the room
// at the same time, plan for that please.
// (Potentially: Switch it to a remote_defense goal)
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_goal(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let cloned_goals = memory.goals.remote_invader_cleanup.clone();

    for goal_room in cloned_goals.keys() {
        achieve_goal(goal_room, memory, cache);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn clear_creeps(goal: &mut RemoteInvaderCleanup) {
    let mut new_creeps = Vec::new();

    for creep in &goal.creeps_assigned {
        let gcreep = game::creeps().get(creep.to_string());

        if let Some(gcreep) = gcreep {
            new_creeps.push(gcreep.name());
        }
    }

    goal.creeps_assigned = new_creeps;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn achieve_goal(target_room: &RoomName, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let goal = memory
        .goals
        .remote_invader_cleanup
        .get_mut(target_room)
        .unwrap();

    clear_creeps(goal);

    let responsible_room = utils::find_closest_owned_room(target_room, cache, Some(2));

    let pos = Position::new(RoomCoordinate::new(15).unwrap(), RoomCoordinate::new(45).unwrap(), *target_room);
    MapVisual::text(pos, format!("👾: {}", goal.destroyed_core), MapTextStyle::default().color("#ff0000").font_size(7.0));

    if let Some(room_cache) = cache.rooms.get_mut(target_room) {
        let invader_core = &room_cache.structures.invader_core;

        if invader_core.is_none() && (room_cache.current_holder != Some("Invader".to_string()) || room_cache.current_holder.is_none()) {
            memory.goals.remote_invader_cleanup.remove(target_room);
            return;
        }

        if invader_core.is_none() {
            goal.destroyed_core = true;
            return;
        } else {
            goal.destroyed_core = false;
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
                .get(responsible_room.room.name())
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
                owning_room: responsible_room.room.name(),
                target_room: Some(*target_room),
                ..CreepMemory::default()
            };

            let name = format!(
                "{}-{}-{}",
                utils::role_to_name(Role::InvaderCoreCleaner),
                responsible_room.room.name(),
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
                responsible_room.room.name(),
                Some(memory),
                None,
                Some(name.clone()),
            );

            goal.creeps_assigned.push(name.clone());

            if let Some(reqs) = cache
                .spawning
                .room_spawn_queue
                .get_mut(&responsible_room.room.name())
            {
                reqs.push(req);
            } else {
                cache
                    .spawning
                    .room_spawn_queue
                    .insert(responsible_room.room.name(), vec![req]);
            }
        } else {
            info!("No responsible room found for {}", target_room);
        }
    }
}
