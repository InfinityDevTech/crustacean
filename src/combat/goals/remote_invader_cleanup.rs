use std::thread::current;

use screeps::{game, Part, RoomName, StructureProperties};

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, room::cache::tick_cache::RoomCache, utils};

pub fn run_goal(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let cloned_goals = memory.goals.remote_invader_cleanup.clone();
    let invader_goals = cloned_goals.keys();

    for goal_room in invader_goals {
        accheive_goal(goal_room, memory, cache);
    }
}

pub fn accheive_goal(target_room: &RoomName, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let goal = memory.goals.remote_invader_cleanup.get_mut(target_room).unwrap();

    if goal.creeps_assigned.is_empty() {
        let responsible_room = utils::find_closest_owned_room(target_room, cache, Some(4));

        if let Some(responsible_room) = responsible_room {
            if let Some(room_cache) = cache.rooms.get_mut(target_room) {
                let hostile_structures = &room_cache.structures.hostile_structures;

                if hostile_structures.is_empty() {
                    memory.goals.remote_invader_cleanup.remove(target_room);
                    return;
                }

                let invader_core = hostile_structures.iter().find(|s| s.structure_type() == screeps::StructureType::InvaderCore);

                if invader_core.is_none() {
                    memory.goals.remote_invader_cleanup.remove(target_room);
                    return;
                }

                let stamp = vec![Part::Attack, Part::Move];
                let stamp_cost = utils::get_body_cost(&stamp);

                let mut body = Vec::new();
                let mut current_cost = 0;

                let responsible_room = cache.rooms.get_mut(&responsible_room).unwrap();
                let available_energy = game::rooms().get(responsible_room.room_name).unwrap().energy_available();

                if available_energy < stamp_cost {
                    if current_cost + available_energy < stamp_cost {
                        return;
                    }

                    body.extend(stamp.clone());
                    current_cost += stamp_cost;
                }

                /*let memory = CreepMemory {
                    role: Role::InvaderCleaner,
                    owning_room: responsible_room.room_name,
                    target_room: Some(*target_room),
                    ..CreepMemory::default()
                };

                let name = format!("{}-{}-{}", Role::InvaderCleaner, responsible_room.room_name, utils::get_unique_id());

                let req = cache.spawning.create_room_spawn_request(Role::InvaderCleaner, body, 4.0, current_cost, responsible_room.room_name, Some(memory), None, Some(name));

                if let Some(reqs) = cache.spawning.room_spawn_queue.get_mut(&responsible_room.room_name) {
                    reqs.push(req);
                } else {
                    cache
                        .spawning
                        .room_spawn_queue
                        .insert(responsible_room.room_name, vec![req]);
                }*/
            }
        }
    }
}