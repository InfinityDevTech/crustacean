use crate::{goal_memory::RemoteInvaderCleanup, memory::ScreepsMemory, room::cache::tick_cache::RoomCache};

pub fn determine_cleanup(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    for (remote_name, remote_memory) in memory.remote_rooms.clone() {
        if memory.goals.remote_invader_cleanup.contains_key(&remote_name) {
            continue;
        }

        if let Some(remote_cache) = cache.rooms.get_mut(&remote_name) {
            if remote_cache.structures.invader_core.is_some() {
                let goal = RemoteInvaderCleanup {
                    cleanup_target: remote_name,
                    creeps_assigned: Vec::new(),
                    destroyed_core: false,
                };

                memory.goals.remote_invader_cleanup.insert(remote_name, goal);
            }
        }
    }
}