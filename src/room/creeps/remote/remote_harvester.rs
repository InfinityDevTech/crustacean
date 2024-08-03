use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{
    game, Creep, HasPosition, Position, ResourceType, RoomCoordinate,
    SharedCreepProperties,
};

use crate::{
    memory::{CreepMemory, ScreepsMemory},
    movement::move_target::MoveOptions,
    room::{
        cache::{{CachedRoom, RoomCache}},
        creeps::local::harvester::{harvest_source, repair_container},
    },
    traits::{
        creep::CreepExtensions, intents_tracking::CreepExtensionsTracking, room::RoomExtensions,
    },
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_remoteharvester(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if let Some(remote_room) = creep_memory.owning_remote {
        if creep_memory.task_id.is_none() {
            creep.bsay("kurt kob", true);
            return;
        }

        if let Some(owning_cache) = cache.rooms.get_mut(&creep_memory.owning_room) {
            if !owning_cache.remotes_with_harvester.contains(&remote_room) {
                owning_cache.remotes_with_harvester.push(remote_room);
            }
        }

        if let Some(remote_room) = cache.rooms.get_mut(&remote_room) {
            remote_room.resources.sources[creep_memory.task_id.unwrap() as usize]
                .add_creep(creep);
        }

        if let Some(remote_room_memory) = memory.remote_rooms.get(&remote_room) {
            if remote_room_memory.under_attack {
                creep.bsay("🚨", false);

                let flee_pos = Position::new(
                    RoomCoordinate::new(25).unwrap(),
                    RoomCoordinate::new(25).unwrap(),
                    creep_memory.owning_room,
                );

                if creep.pos().get_range_to(flee_pos) > 23 {
                    let or = creep_memory.owning_room;
                    creep.better_move_to(
                        memory,
                        cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
                        Position::new(
                            RoomCoordinate::new(25).unwrap(),
                            RoomCoordinate::new(25).unwrap(),
                            or,
                        ),
                        23,
                        MoveOptions::default(),
                    );

                    return;
                }

                return;
            }
        }

        if creep.tired() || creep.spawning() || game::cpu::bucket() < 1000 {
            creep.bsay("😴", false);
            return;
        }

        let room_name = creep.room().unwrap().name();
        let room_cache = cache.rooms.get_mut(&room_name).unwrap();

        if room_name != remote_room {
            if let Some(remote_room_memory) = memory.remote_rooms.get(&remote_room) {

            }

            creep.bsay("🚚", false);

            let x = unsafe { RoomCoordinate::unchecked_new(25) };
            let y = unsafe { RoomCoordinate::unchecked_new(25) };

            let position = Position::new(x, y, remote_room);

            creep.better_move_to(
                memory,
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
                    creep.bsay("kurt kob", true);
                }
            }

            if let Some(remote_room_memory) = memory.remote_rooms.get_mut(&remote_room) {
                if remote_room_memory.under_attack {
                    let flee_pos = Position::new(
                        RoomCoordinate::new(25).unwrap(),
                        RoomCoordinate::new(25).unwrap(),
                        creep_memory.owning_room,
                    );

                    if creep.pos().get_range_to(flee_pos) > 24 {
                        creep.better_move_to(memory, room_cache, flee_pos, 24, MoveOptions::default().avoid_enemies(true));
    
                        return;
                    } else {
                        creep.bsay("🚨", false);

                        return;
                    }
                }
            }

            let scouted_source =
                &mut room_cache.resources.sources[creep_memory.task_id.unwrap() as usize];
            scouted_source.add_creep(creep);

            let mut source = scouted_source.clone();

            if creep.store().get_used_capacity(None) as f32
                >= (creep.store().get_capacity(None) as f32 * 0.5)
            {
                deposit_enegy(creep, memory, room_cache);
            }

            let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
            let owning_room = creep_memory.owning_room;

            let harvested_amount = harvest_source(creep, &mut source, memory, room_cache);
            if let Some(harvested_amount) = harvested_amount {
                if let Some(remote_room) = cache.rooms.get_mut(&owning_room) {
                    remote_room.stats.energy.income_energy += harvested_amount;
                }
            }
        }
    } else {
        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

        let mut rng = StdRng::seed_from_u64(game::time() as u64);
        let remotes = memory
            .rooms
            .get(&creep_memory.owning_room)
            .unwrap()
            .remotes
            .clone();

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
    let source = &cache.resources.sources[creep_memory.task_id.unwrap() as usize].source;

    for csite in cache.structures.construction_sites() {
        if csite.structure_type() == screeps::StructureType::Container
            && csite.pos().is_near_to(source.pos())
        {
            let _ = creep.ITbuild(csite);
            return true;
        }
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn deposit_enegy(creep: &Creep, memory: &mut ScreepsMemory, remote_cache: &mut CachedRoom) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let contianer = &remote_cache.resources.sources[creep_memory.task_id.unwrap() as usize].container.clone();

    if build_container(creep, creep_memory, remote_cache) {
        return;
    }

    if let Some(contianer) = contianer {
        if repair_container(creep, memory, remote_cache, contianer) {
            return;
        }

        if contianer
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            // Why am I wasting the CPU to drop it?
            // It will automatically drop and not cost me the 0.2 CPU.
            //let _ = creep.ITdrop(ResourceType::Energy, None);
        } else if creep.pos().get_range_to(contianer.pos()) > 1 {
            creep.better_move_to(
                memory,
                remote_cache,
                contianer.pos(),
                1,
                MoveOptions::default(),
            );
        } else {
            let _ = creep.ITtransfer(contianer, ResourceType::Energy, None);
        }
    } else {
        // Auto drop, save 0.2 CPU.
        //let _ = creep.ITdrop(ResourceType::Energy, None);
    }
}
