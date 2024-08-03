use screeps::{find, game, HasId, HasPosition, OwnedStructureProperties, Position, Room, RoomXY, StructureObject};

use crate::{
    memory::{EnemyPlayer, ScoutedRoom, ScoutedSource, ScreepsMemory}, room::cache::CachedRoom, traits::position::PositionExtensions, utils
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn scout_room(room: &Room, memory: &mut ScreepsMemory, cached_room: &mut CachedRoom) {
    if memory.rooms.contains_key(&room.name()) {
        return;
    }

    let room_name = room.name();

    let mut room_rcl = None;
    let mut owner = None;
    let mut reserved = None;

    if let Some(controller) = room.controller() {
        room_rcl = Some(controller.level());

        if controller.owner().is_some() {
            owner = Some(controller.owner().unwrap().username())
        }

        if controller.reservation().is_some() {
            reserved = Some(controller.reservation().unwrap().username())
        }
    }

    let mut invader_owned = None;
    for structure in room.find(find::HOSTILE_STRUCTURES, None) {
        if let StructureObject::StructureInvaderCore(invader) = structure {
            if invader.level() > 0 {
                invader_owned = Some(true);
                break;
            }
        }
    }

    let mineral_id = if cached_room.resources.mineral.is_some() {
        Some(cached_room.resources.mineral.as_ref().unwrap().id())
    } else {
        None
    };

    let sources: Vec<RoomXY> = cached_room.resources.sources.iter().map(|x| x.source.pos().xy()).collect();
    let sources = if sources.is_empty() {
        None
    } else {
        let mut cached_sources = Vec::new();
        for source in sources {
            let pos = Position::new(source.x, source.y, room_name);
            cached_sources.push(ScoutedSource {
                pos: source,
                pos_av: pos.get_accessible_positions_around(1).len() as u8,
            });
        }

        Some(cached_sources)
    };

    let scouted_room = ScoutedRoom {
        name: room_name,
        room_type: utils::room_type(&room_name),
        rcl: room_rcl,
        owner: owner.clone(),
        invader_core: invader_owned,
        reserved: reserved.clone(),
        defense_capability: 0,
        sources: sources.clone(),
        mineral: mineral_id,
        last_scouted: game::time(),
    };

    if owner.is_some() {
        if let std::collections::hash_map::Entry::Vacant(e) = memory
            .enemy_players
            .entry(owner.clone().unwrap().to_string())
        {
            let enemy = EnemyPlayer {
                username: owner.clone().unwrap().to_string(),
                hate: 0.0,
                owned_rooms: vec![room_name],
                reserved_rooms: vec![],
                last_attack: 0,
            };

            e.insert(enemy);
        } else {
            let enemy_player = memory
                .enemy_players
                .get_mut(&owner.clone().unwrap().to_string())
                .unwrap();

            if !enemy_player.owned_rooms.contains(&room_name) {
                enemy_player.owned_rooms.push(room_name);
            }
        }
    }

    if reserved.is_some() {
        if let std::collections::hash_map::Entry::Vacant(e) = memory
            .enemy_players.entry(reserved.clone().unwrap().to_string()) {
            let enemy = EnemyPlayer {
                username: reserved.clone().unwrap().to_string(),
                hate: 0.0,
                owned_rooms: vec![],
                reserved_rooms: vec![room_name],
                last_attack: 0,
            };

            e.insert(enemy);
        } else {
            let enemy_player = memory
                .enemy_players
                .get_mut(&reserved.clone().unwrap().to_string())
                .unwrap();

            if !enemy_player.reserved_rooms.contains(&room_name) {
                enemy_player.reserved_rooms.push(room_name);
            }
        }
    }

    if let std::collections::hash_map::Entry::Vacant(e) = memory.scouted_rooms.entry(room_name) {
        e.insert(scouted_room);
    } else {
        let _scouted = memory.scouted_rooms.remove(&room_name);

        memory.scouted_rooms.insert(room_name, scouted_room);
    }
}
