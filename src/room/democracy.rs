use std::{collections::HashMap, str::FromStr};

use log::info;
use screeps::{
    find, game, look::{self, LookResult}, HasId, HasPosition, ObjectId, Part, Room, Terrain
};

use crate::{memory::{ScoutedSource, ScreepsMemory}, traits::room::RoomExtensions};

use super::{creeps, planning::creep::miner::formulate_miner, tower};

pub fn start_government(room: Room, memory: &mut ScreepsMemory) {
    info!("[GOVERNMENT] Starting government for room: {}", room.name());
    let spawn = room.find(find::MY_SPAWNS, None).pop().expect("Failed to find spawn in room");

    let (cost, body) = formulate_miner(&room);

    info!("Cost: {}", cost);
    info!("Body: {:?}", room.energy_available());

    if cost < room.energy_available() {
        info!("Building");
        info!("{:?}", spawn.spawn_creep(&body, "Minors"));
    }

    tower::run_towers(&room);
    creeps::organizer::run_creeps(&room, memory);
}
