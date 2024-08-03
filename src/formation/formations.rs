use crate::{memory::ScreepsMemory, room::cache::RoomCache};

use super::duo;

pub fn run_formations(memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    duo::run_duo::run_duos(memory, cache);
}