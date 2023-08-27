use screeps::{Room, Part, find};

use crate::memory::{ScreepsMemory, Task};

pub fn create_miner(memory: &mut ScreepsMemory, room: Room) -> bool {
    if memory.rooms.get(&room.name().to_string()).unwrap().c_c.miner >= 1 && memory.rooms.get(&room.name().to_string()).unwrap().c_c.hauler < 1 {return false;}
    let sources = memory.clone().rooms.get(&room.name().to_string()).unwrap().mine.clone();
    let mut selected_source = None;
    for (source_id, source_mem) in sources {
        if selected_source.is_none() && source_mem.s > source_mem.u {
            selected_source = Some(source_id);
        } else {
            continue;
        }
    }
    if let Some(source) = selected_source {
        let name = format!("m-{}", memory.rooms.get(&room.name().to_string()).unwrap().c_m);
        let body = [Part::Move, Part::Work, Part::Work];
        let spawn_res = room.find(find::MY_SPAWNS, None).first().unwrap().spawn_creep(&body, &name);
        if spawn_res.is_ok() {
            memory.create_creep(
                &room.name().to_string(),
                &name,
                crate::memory::Careers::Mining,
                Some(Task::Miner(source)),
            );
            memory.rooms.get_mut(&room.name().to_string()).unwrap().c_c.miner += 1;
            memory.rooms.get_mut(&room.name().to_string()).unwrap().c_m += 1;
            memory.rooms.get_mut(&room.name().to_string()).unwrap().mine.get_mut(&source).unwrap().u += 1;
            true
        } else {
            false
        }
    } else {
        false
    }
}

pub fn miner_died(memory: &mut ScreepsMemory, name: &str, room: &str) {
    // Remove from rooms creep count and from room creep list
    memory.rooms.get_mut(room).unwrap().c_c.miner -= 1;
    memory.rooms.get_mut(room).unwrap().cs = memory
        .rooms
        .get_mut(room)
        .unwrap()
        .cs
        .iter()
        .filter(|x| x != &&name.to_string())
        .map(|x| x.to_string())
        .collect();

    // Downtick the counters for used sources
    let mining_source_id = memory.creeps.get(name).unwrap().t.clone().unwrap();
    if let Task::Miner(source_id) = mining_source_id {
        memory
            .rooms
            .get_mut(room)
            .unwrap()
            .mine
            .get_mut(&source_id)
            .unwrap()
            .u -= 1;
    }

        // Remove said creep from memory
        memory.creeps.remove(name);
}

pub fn hauler_died(memory: &mut ScreepsMemory, name: &str, room: &str) {
    // Remove from rooms creep count and from room creep list
    memory.rooms.get_mut(room).unwrap().c_c.hauler -= 1;
    memory.rooms.get_mut(room).unwrap().cs = memory
        .rooms
        .get_mut(room)
        .unwrap()
        .cs
        .iter()
        .filter(|x| x != &&name.to_string())
        .map(|x| x.to_string())
        .collect();

    // Remove said creep from memory
    memory.creeps.remove(name);
}

pub fn upgrader_died(memory: &mut ScreepsMemory, name: &str, room: &str) {
    // Remove from rooms creep count and from room creep list
    memory.rooms.get_mut(room).unwrap().c_c.hauler -= 1;
    memory.rooms.get_mut(room).unwrap().cs = memory
        .rooms
        .get_mut(room)
        .unwrap()
        .cs
        .iter()
        .filter(|x| x != &&name.to_string())
        .map(|x| x.to_string())
        .collect();

    // Remove said creep from memory
    memory.creeps.remove(name);
}