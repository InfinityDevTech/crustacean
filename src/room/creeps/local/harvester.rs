use log::info;
use screeps::{
    Creep, HasHits, HasPosition, Part, ResourceType, SharedCreepProperties, Source,
    StructureContainer,
};

use crate::{
    memory::ScreepsMemory,
    movement::move_target::MoveOptions,
    room::cache::{resources::CachedSource, CachedRoom, RoomCache},
    traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking, room::RoomExtensions},
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_harvester(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    if creep_memory.task_id.is_none() {
        let task = creep.room().unwrap().get_target_for_miner(cached_room);

        if let Some(task) = task {
            creep_memory.task_id = Some(task.into());
        } else {
            creep.bsay("kurt kob", true);
            info!("Harvester {} has no task id", creep.name());
            let _ = creep.ITsuicide();
            return;
        }
    }

    let pointer_index = creep_memory.task_id.unwrap() as usize;
    let scouted_source = &mut cached_room.resources.sources[pointer_index];
    scouted_source.add_creep(creep);

    let mut source = scouted_source.clone();

    if creep.spawning() || creep.tired() {
        creep.bsay("😴", false);
        return;
    }

    if creep.store().get_free_capacity(None) as f32
        <= (creep.store().get_capacity(None) as f32 * 0.5)
    {
        if !deposit_energy(creep, memory, cached_room) {
            harvest_source(creep, &mut source, memory, cached_room);
        }
    } else {
        harvest_source(creep, &mut source, memory, cached_room);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn harvest_source(
    creep: &Creep,
    source: &mut CachedSource,
    memory: &mut ScreepsMemory,
    cache: &mut CachedRoom,
) -> Option<u32> {
    if !creep.pos().is_near_to(source.source.pos()) {
        creep.bsay("🚚 🔋", false);

        let open_pos = source.get_best_pos_to_stand(&cache.creeps.creeps_at_pos);

        if let Some(pos) = open_pos {
            creep.better_move_to(memory, cache, pos, 0, MoveOptions::default());
        } else {
            // If the source is flooded, let the traffic manager figure it out.
            //creep.better_move_to(memory, cache, source.source.pos(), 1, MoveOptions::default());
        }

        None
    } else {
        if source.source.energy() == 0 {
            return None;
        }
        creep.bsay("⛏️", false);
        let _ = creep.ITharvest(&source.source);

        let amount_harvsted = get_aproximate_energy_mined(creep, &source.source);
        cache.stats.energy.income_energy += amount_harvsted;

        Some(amount_harvsted)
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn deposit_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut CachedRoom) -> bool {
    if creep.store().get_used_capacity(None) == 0 {
        return false;
    }

    creep.bsay("📦", false);

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let source = &cache.resources.sources[creep_memory.task_id.unwrap() as usize];

    if let Some(container) = &source.container {
        if (cache.rcl < 4 || source.link.is_none()) && container.hits() < container.hits_max() {
            if container.pos().get_range_to(creep.pos()) > 1 {
                creep.bsay("🚚", false);
                creep.better_move_to(memory, cache, container.pos(), 1, MoveOptions::default());
                return true;
            } else {
                creep.bsay("🔧", false);
                let _ = creep.ITrepair(container);
                return true;
            }
        }
    }

    let task_id = creep_memory.task_id.unwrap() as usize;

    if let Some(link) = &source.link.clone() {
        if creep.pos().is_near_to(link.pos()) {
            let _ = creep.ITtransfer(link, ResourceType::Energy, None);
            return false;
        } else {
            creep.better_move_to(memory, cache, link.pos(), 1, MoveOptions::default());
            return true;
        }
    }

    if let Some(container) = &cache.resources.sources[task_id].container.clone() {
        if repair_container(creep, memory, cache, container) {
            return true;
        }

        if creep.pos() == container.pos() {
            // Why am I wasting the CPU to drop it?
            // It will automatically drop and not cost me the 0.2 CPU.
            //let amount = creep.store().get_used_capacity(Some(ResourceType::Energy));

            //let _ = creep.ITdrop(ResourceType::Energy, Some(amount));
            false
        } else if creep.pos().is_near_to(container.pos()) {
            let _ = creep.ITtransfer(container, ResourceType::Energy, None);

            return false;
        } else {
            creep.better_move_to(memory, cache, container.pos(), 1, MoveOptions::default());

            return true;
        }
    } else {
        // Auto drop, save 0.2 CPU
        //let _ = creep.ITdrop(ResourceType::Energy, None);
        false
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn repair_container(
    creep: &Creep,
    memory: &mut ScreepsMemory,
    cache: &mut CachedRoom,
    container: &StructureContainer,
) -> bool {
    if creep.store().get_used_capacity(None) == 0 {
        return false;
    }

    if cache.rcl >= 4 {
        return false;
    }

    if container.hits() < container.hits_max() {
        if container.pos().get_range_to(creep.pos()) > 1 {
            creep.bsay("🚚", false);
            creep.better_move_to(memory, cache, container.pos(), 1, MoveOptions::default());
            return true;
        } else {
            creep.bsay("🔧", false);
            let _ = creep.ITrepair(container);
            return true;
        }
    }
    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_aproximate_energy_mined(creep: &Creep, source: &Source) -> u32 {
    let work_parts = creep
        .body()
        .iter()
        .filter(|x| x.part() == Part::Work && x.hits() > 0)
        .count() as u32;

    let max_mineable = work_parts * 2;
    let source_remaining = source.energy();

    std::cmp::min(max_mineable, source_remaining)
}
