use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{game, Creep, HasPosition, MaybeHasId, Position, RoomCoordinate, SharedCreepProperties};

use crate::{
    memory::{CreepMemory, ScreepsMemory},
    room::{
        cache::tick_cache::{CachedRoom, RoomCache},
        creeps::local::source_miner,
    },
    traits::{creep::CreepExtensions, room::RoomExtensions},
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    if let Some(remote_room) = memory.creeps.get(&creep.name()).unwrap().owning_remote {
        if creep.room().unwrap().name() != remote_room {
            let _ = creep.say("🚚", false);

            let x = unsafe { RoomCoordinate::unchecked_new(25) };
            let y = unsafe { RoomCoordinate::unchecked_new(25) };

            let position = Position::new(x, y, remote_room);

            creep.better_move_to(
                memory.creeps.get_mut(&creep.name()).unwrap(),
                cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
                position,
                24,
            );
        } else {
            let room = game::rooms().get(remote_room).unwrap();

            cache.create_if_not_exists(&room, memory);
            let room_cache = cache.rooms.get_mut(&remote_room).unwrap();
            let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

            if creep_memory.task_id.is_none() {
                let task = room.get_target_for_miner(room_cache);

                if let Some(task) = task {
                    creep_memory.task_id = Some(task.into());
                } else {
                    let _ = creep.say("kurt kob", true);
                }
            }

            let scouted_source = &mut room_cache.resources.sources[creep_memory.task_id.unwrap() as usize];
            scouted_source.creeps.push(creep.try_id().unwrap());

            if !build_container(creep, creep_memory, room_cache) || !source_miner::repair_container(creep, creep_memory, room_cache) {
                let source = game::get_object_by_id_typed(
                    &room_cache.resources.sources[creep_memory.task_id.unwrap() as usize]
                        .id,
                )
                .unwrap();
                source_miner::harvest_source(creep, source, creep_memory, room_cache);

                if creep.store().get_free_capacity(None) as f32
                    <= (creep.store().get_capacity(None) as f32 * 0.5)
                {
                    source_miner::deposit_energy(creep, creep_memory, room_cache);
                }
            }
        }
    } else {
        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

        let mut rng = StdRng::seed_from_u64(game::time() as u64);
        let remotes = memory.rooms.get(&creep_memory.owning_room).unwrap().remotes.clone();

        let remote = remotes[rng.gen_range(0..remotes.len())];

        creep_memory.owning_remote = Some(remote);
    }
}

pub fn build_container(
    creep: &Creep,
    creep_memory: &mut CreepMemory,
    cache: &mut CachedRoom,
) -> bool {
    let source = game::get_object_by_id_typed(
        &cache.resources.sources[creep_memory.task_id.unwrap() as usize]
            .id
            .clone(),
    )
    .unwrap();

    for csite in &cache.structures.construction_sites {
        if csite.structure_type() == screeps::StructureType::Container
            && csite.pos().is_near_to(source.pos())
        {
            let _ = creep.build(csite);
            return true;
        }
    }

    false
}

pub fn find_remote_task(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    let remotes = memory.rooms.get(&creep_memory.owning_room).unwrap().remotes.clone();

    let mut rng = StdRng::seed_from_u64(game::time() as u64);

    let remote = remotes[rng.gen_range(0..remotes.len())];

    creep_memory.owning_remote = Some(remote);
}