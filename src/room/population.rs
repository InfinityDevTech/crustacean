use screeps::{find, Part, Room};

use crate::{
    memory::{ScreepsMemory, Task},
    traits::room::RoomExtensions,
};

pub fn create_miner(memory: &mut ScreepsMemory, room: Room) -> bool {
    if !memory.get_room(&room.name_str()).get_creeps_by_role("miner").is_empty() && memory.get_room(&room.name_str()).get_creeps_by_role("hauler").is_empty() {
        return false;
    }

    let sources = memory.get_room(&room.name_str()).mine.clone();
    let mut selected_source = None;
    for (source_id, source_mem) in sources {
        if selected_source.is_none() && source_mem.s > source_mem.u {
            selected_source = Some(source_id);
        } else {
            continue;
        }
    }
    if let Some(source) = selected_source {
        let name = format!("m-{}", memory.get_room(&room.name_str()).creeps_made);
        let body = [Part::Move, Part::Work, Part::Work];
        let spawn_res = room
            .find(find::MY_SPAWNS, None)
            .first()
            .unwrap()
            .spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                &room.name_str(),
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Miner(source)),
            );
            memory.get_room(&room.name_str()).creeps.insert(name, "miner".to_string());
            memory.get_room(&room.name_str()).creeps_made += 1;
            memory.get_room(&room.name_str()).mine.get_mut(&source).unwrap().u += 1;
            memory.stats.rooms.get_mut(&room.name_str()).unwrap().creeps_made += 1;
            true
        } else {
            false
        }
    } else {
        false
    }
}

pub fn miner_died(memory: &mut ScreepsMemory, creep_name: &str, room_name: &str) {
    // Remove from rooms creep count and from room creep list
    memory.get_room(room_name).creeps.remove(&creep_name.to_string());

    // Downtick the counters for used sources
    let mining_source_id = memory
        .get_creep(creep_name)
        .t
        .clone()
        .expect("Failed to get creep task from memory");

    if let Task::Miner(source_id) = mining_source_id {
        memory
            .get_room(room_name)
            .mine
            .get_mut(&source_id)
            .expect("Failed to get source from memory")
            .u -= 1;
    }

    // Remove said creep from memory
    memory.creeps.remove(creep_name);
}
