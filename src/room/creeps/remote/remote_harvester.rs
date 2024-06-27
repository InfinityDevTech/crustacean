use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{game, Creep, HasPosition, MaybeHasId, Position, ResourceType, RoomCoordinate, SharedCreepProperties};

use crate::{
    memory::{CreepMemory, ScreepsMemory}, movement::move_target::MoveOptions, room::{
        cache::tick_cache::{CachedRoom, RoomCache},
        creeps::local::source_miner::{harvest_source, repair_container},
    }, traits::{creep::CreepExtensions, room::RoomExtensions}
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_remoteharvester(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if let Some(remote_room) = creep_memory.owning_remote {

        if creep_memory.task_id.is_none() {
            let _ = creep.say("kurt kob", true);
            return;
        }

        if let Some(remote_room) = cache.rooms.get_mut(&remote_room) {
            remote_room.resources.sources[creep_memory.task_id.unwrap() as usize].creeps.push(creep.try_id().unwrap());
        }

        if creep.tired() {
            let _ = creep.say("😴", false);
            return;
        }

        let room_name = creep.room().unwrap().name();
        let room_cache = cache.rooms.get_mut(&room_name).unwrap();

        if room_name != remote_room {
            let _ = creep.say("🚚", false);

            let x = unsafe { RoomCoordinate::unchecked_new(25) };
            let y = unsafe { RoomCoordinate::unchecked_new(25) };

            let position = Position::new(x, y, remote_room);

            creep.better_move_to(
                creep_memory,
                cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
                position,
                24,
                MoveOptions::default(),
            );
        } else {
            let room = game::rooms().get(remote_room).unwrap();

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

            let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

            if creep.store().get_used_capacity(None) as f32 >= (creep.store().get_capacity(None) as f32 * 0.5) {
                deposit_enegy(creep, creep_memory, room_cache);
            }

            harvest_source(creep, source, creep_memory, room_cache);
        }
    } else {
        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

        let mut rng = StdRng::seed_from_u64(game::time() as u64);
        let remotes = memory.rooms.get(&creep_memory.owning_room).unwrap().remotes.clone();

        let remote = remotes[rng.gen_range(0..remotes.len())];

        creep_memory.owning_remote = Some(remote);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn deposit_enegy(creep: &Creep, creep_memory: &mut CreepMemory, remote_cache: &mut CachedRoom) {
    let contianer = &remote_cache.resources.sources[creep_memory.task_id.unwrap() as usize].get_container(&remote_cache.structures);

    if build_container(creep, creep_memory, remote_cache) {
        return;
    }

    if let Some(contianer) = contianer {
        if repair_container(creep, creep_memory, remote_cache, contianer) {
            return;
        }

        if contianer.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            let _ = creep.drop(ResourceType::Energy, None);
        } else if creep.pos().get_range_to(contianer.pos()) > 1 {
            creep.better_move_to(creep_memory, remote_cache, contianer.pos(), 1, MoveOptions::default());
        } else {
            let _ = creep.transfer(contianer, ResourceType::Energy, None);
        }
    } else {
        let _ = creep.drop(ResourceType::Energy, None);
    }
}