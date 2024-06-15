use std::str::FromStr;

use screeps::{game, RoomName, SharedCreepProperties};

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, utils::name_to_role};

pub fn recover_creeps(memory: &mut ScreepsMemory) {
    let creep_names = game::creeps().keys();
    for creep_name in creep_names {
        if memory.creeps.contains_key(&creep_name) {
            continue;
        }

        let mut split_name = creep_name.split('-');
        let role = name_to_role(split_name.next().unwrap());
        let room = split_name.next().unwrap();
        let _id = split_name.next().unwrap();

        let room_name = RoomName::from_str(room);

        // This fixes an issue with the old naming convention
        // Past - <ROLE>-<GAME_TIME>-<ID>
        // New - <ROLE>-<ROOM>-<ID>
        if room_name.is_err() {
            let _ = game::creeps().get(creep_name.clone()).unwrap().suicide();

            return;
        }
        let room_name = room_name.unwrap();

        let creep = game::creeps().get(creep_name.clone()).unwrap();

        let Some(role) = role else {
            let _ = creep.suicide();
            continue;
        };

        match role {
            Role::Hauler => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Hauler, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::Builder => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Builder, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::Upgrader => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Upgrader, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::Scout => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Scout, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::Bulldozer => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Bulldozer, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            _ => {
                let _ = creep.suicide();
            }
        }
    }
}