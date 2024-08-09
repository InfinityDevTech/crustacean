use std::str::FromStr;

use screeps::{game, RoomName, SharedCreepProperties};

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, traits::intents_tracking::CreepExtensionsTracking, utils::name_to_role};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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
            let _ = game::creeps().get(creep_name.clone()).unwrap().ITsuicide();

            return;
        }
        let room_name = room_name.unwrap();

        let creep = game::creeps().get(creep_name.clone()).unwrap();

        let Some(role) = role else {
            let _ = creep.ITsuicide();
            continue;
        };

        match role {
            Role::Harvester => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Harvester, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::MineralMiner => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::MineralMiner, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::RemoteHarvester => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::RemoteHarvester, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::FastFiller => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::FastFiller, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::Hauler => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Hauler, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::Builder => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Builder, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            Role::Repairer => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::Repairer, ..Default::default() };

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
            Role::BaseHauler => {
                let cmemory = CreepMemory { owning_room: room_name, role: Role::BaseHauler, ..Default::default() };

                memory.create_creep(&room_name, &creep.name(), cmemory);
            }
            _ => {
                // TODO: Make this find the closest room to the creep and assign it to that room
                // That way we can recycle it.
                let _ = creep.ITsuicide();
            }
        }
    }
}