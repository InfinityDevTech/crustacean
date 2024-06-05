use screeps::{control, game, HasId, OwnedStructureProperties, Room};

use crate::{
    memory::{EnemyPlayer, ScoutedRoom, ScreepsMemory},
    room::{self, cache::tick_cache::RoomCache},
};

pub fn rank_room(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if let Some(controller) = room.controller() {
        let room_name = room.name();
        let room_rcl = controller.level();

        let owner: Option<String> = if controller.owner().is_some() {
            Some(controller.owner().unwrap().username())
        } else {
            None
        };
        let reserved: Option<String> = if controller.reservation().is_some() {
            Some(controller.reservation().unwrap().username())
        } else {
            None
        };

        let mineral_id = if cache.resources.mineral.is_some() {
            Some(cache.resources.mineral.as_ref().unwrap().id())
        } else {
            None
        };

        let mut scouted_room = ScoutedRoom {
            name: room_name,
            rcl: room_rcl,
            owner: owner.clone(),
            reserved: reserved.clone(),
            defense_capability: 0,
            sources: cache.resources.sources.len() as u8,
            mineral: mineral_id,
            last_scouted: game::time(),
        };

        if owner.is_some() {
            if let std::collections::hash_map::Entry::Vacant(e) = memory.enemy_players.entry(owner.clone().unwrap().to_string()) {
                let enemy = EnemyPlayer {
                    username: owner.clone().unwrap().to_string(),
                    hate: 0.0,
                    owned_rooms: vec![room_name],
                    reserved_rooms: vec![],
                    last_attack: 0,
                };

                e.insert(enemy);
            } else {
                let enemy_player = memory.enemy_players.get_mut(&owner.clone().unwrap().to_string()).unwrap();

                if !enemy_player.owned_rooms.contains(&room_name) {
                    enemy_player.owned_rooms.push(room_name);
                }
            }
        }

        if reserved.is_some() {
            if memory.enemy_players.contains_key(&reserved.clone().unwrap().to_string()) {
                let enemy_player = memory.enemy_players.get_mut(&owner.clone().unwrap().to_string()).unwrap();

                if !enemy_player.reserved_rooms.contains(&room_name) {
                    enemy_player.reserved_rooms.push(room_name);
                }
            }  else {
                let enemy = EnemyPlayer {
                    username: owner.clone().unwrap().to_string(),
                    hate: 0.0,
                    owned_rooms: vec![],
                    reserved_rooms: vec![room_name],
                    last_attack: 0,
                };

                memory.enemy_players.insert(owner.clone().unwrap().to_string(), enemy);
            }
        }

        if let std::collections::hash_map::Entry::Vacant(e) = memory.scouted_rooms.entry(room_name) {
            e.insert(scouted_room);
        } else {
            let scouted = memory.scouted_rooms.get_mut(&room_name).unwrap();
            scouted.rcl = room_rcl;
            scouted.owner.clone_from(&owner);
            scouted.reserved = reserved;
            scouted.defense_capability = 0;
            scouted.sources = cache.resources.sources.len() as u8;
            scouted.mineral = mineral_id;
            scouted.last_scouted = game::time();
        }
    }
}