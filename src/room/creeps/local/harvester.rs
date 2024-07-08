use screeps::{
    game, Creep, HasHits, HasPosition, MaybeHasId, Part, ResourceType, SharedCreepProperties,
    Source, StructureContainer,
};

use crate::{
    memory::{CreepMemory, ScreepsMemory},
    movement::move_target::MoveOptions,
    room::cache::tick_cache::{CachedRoom, RoomCache},
    traits::creep::CreepExtensions,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_harvester(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if creep_memory.task_id.is_none() {
        creep.bsay("kurt kob", true);
        let _ = creep.suicide();
        return;
    }

    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    let pointer_index = creep_memory.task_id.unwrap() as usize;
    let scouted_source = &mut cached_room.resources.sources[pointer_index];
    scouted_source.creeps.push(creep.try_id().unwrap());

    let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

    if creep.spawning() || creep.tired() {
        creep.bsay("😴", false);
        return;
    }

    if creep.store().get_free_capacity(None) as f32
        <= (creep.store().get_capacity(None) as f32 * 0.5)
    {
        if !link_deposit(creep, creep_memory, cached_room) {
            deposit_energy(creep, creep_memory, cached_room);
        }
        harvest_source(creep, source, creep_memory, cached_room);
    } else {
        harvest_source(creep, source, creep_memory, cached_room);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn harvest_source(
    creep: &Creep,
    source: Source,
    creep_memory: &mut CreepMemory,
    cache: &mut CachedRoom,
) -> Option<u32> {
    if !creep.pos().is_near_to(source.pos()) {
        creep.bsay("🚚 🔋", false);

        creep.better_move_to(creep_memory, cache, source.pos(), 1, MoveOptions::default());

        None
    } else {
        creep.bsay("⛏️", false);
        let _ = creep.harvest(&source);

        let amount_harvsted = get_aproximate_energy_mined(creep, &source);
        cache.stats.energy.income_energy += amount_harvsted;

        Some(amount_harvsted)
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn link_deposit(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut CachedRoom) -> bool {
    let link_id =
        cache.resources.sources[creep_memory.task_id.unwrap() as usize].get_link(&cache.structures);

    if let Some(link) = link_id {
        if creep.pos().is_near_to(link.pos()) {
            creep.bsay("🔗", false);
            let _ = creep.transfer(
                &link,
                ResourceType::Energy,
                Some(creep.store().get_used_capacity(Some(ResourceType::Energy))),
            );
        } else {
            return false;
        }
    }
    false
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn deposit_energy(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut CachedRoom) {
    creep.bsay("📦", false);

    let source = &cache.resources.sources[creep_memory.task_id.unwrap() as usize];

    if let Some(link) = source.link {
        if let Some(container) = cache.resources.sources[creep_memory.task_id.unwrap() as usize]
            .get_container(&cache.structures)
        {
            if repair_container(creep, creep_memory, cache, &container) {
                return;
            }
        }

        let link = game::get_object_by_id_typed(&link).unwrap();

        if link.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
            if creep.pos().is_near_to(link.pos()) {
                let _ = creep.transfer(
                    &link,
                    ResourceType::Energy,
                    Some(creep.store().get_used_capacity(Some(ResourceType::Energy))),
                );
            } else {
                creep.better_move_to(creep_memory, cache, link.pos(), 1, MoveOptions::default());
            }
            return;
        }
    }

    if let Some(container) = cache.resources.sources[creep_memory.task_id.unwrap() as usize]
        .get_container(&cache.structures)
    {
        if repair_container(creep, creep_memory, cache, &container) {
            return;
        }

        if container
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            let amount = creep.store().get_used_capacity(Some(ResourceType::Energy));

            let _ = creep.drop(ResourceType::Energy, Some(amount));
        } else if creep.pos().is_near_to(container.pos()) {
            let _ = creep.transfer(&container, ResourceType::Energy, None);
        } else {
            creep.better_move_to(
                creep_memory,
                cache,
                container.pos(),
                1,
                MoveOptions::default(),
            );
        }
    } else {
        let _ = creep.drop(ResourceType::Energy, None);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn repair_container(
    creep: &Creep,
    creep_memory: &mut CreepMemory,
    cache: &mut CachedRoom,
    container: &StructureContainer,
) -> bool {
    if creep.store().get_used_capacity(None) == 0 {
        return false;
    }

    if container.hits() < container.hits_max() {
        if container.pos().get_range_to(creep.pos()) > 1 {
            creep.bsay("🚚", false);
            creep.better_move_to(
                creep_memory,
                cache,
                container.pos(),
                1,
                MoveOptions::default(),
            );
            return true;
        } else {
            creep.bsay("🔧", false);
            let _ = creep.repair(container);
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
