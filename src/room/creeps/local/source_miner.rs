use screeps::{
    game, Creep, HasHits, HasPosition, MaybeHasId, Part, ResourceType, SharedCreepProperties, Source
};

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::tick_cache::{
        hauling::{HaulingPriority, HaulingType}, CachedRoom, RoomCache
    },
    traits::creep::CreepExtensions, utils::scale_haul_priority,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if creep_memory.task_id.is_none() {
        let _ = creep.say("kurt kob", true);
        let _ = creep.suicide();
        return;
    }

    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    let pointer_index = creep_memory.task_id.unwrap() as usize;
    let scouted_source = &mut cached_room.resources.sources[pointer_index];
    scouted_source.creeps.push(creep.try_id().unwrap());

    let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
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

pub fn harvest_source(creep: &Creep, source: Source, creep_memory: &mut CreepMemory, cache: &mut CachedRoom) {
    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
        return;
    }

    if !creep.pos().is_near_to(source.pos()) {
        let _ = creep.say("🚚 🔋", false);
        creep.better_move_to(creep_memory, cache, source.pos(), 1);
    } else {
        let _ = creep.say("⛏️", false);
        let _ = creep.harvest(&source);

        let amount_harvsted = get_aproximate_energy_mined(creep, &source);
        cache.stats.energy.income_mining += amount_harvsted;
    }
}

fn link_deposit(creep: &Creep, creep_memory: &mut CreepMemory, cache: &CachedRoom) -> bool {
    let link_id = creep_memory.link_id;

    if let Some(linkid) = link_id {
        let link = cache.structures.links.get(&linkid).unwrap();

        if creep.pos().is_near_to(link.pos()) {
            let _ = creep.say("🔗", false);
            let _ = creep.transfer(
                link,
                ResourceType::Energy,
                Some(creep.store().get_used_capacity(Some(ResourceType::Energy))),
            );
        } else {
            return false;
        }
    }
    false
}

pub fn deposit_energy(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut CachedRoom) {
    if repair_container(creep, creep_memory, cache) {
        return;
    }
    let _ = creep.say("📦", false);

    if let Some(container) =
        cache.resources.sources[creep_memory.task_id.unwrap() as usize].get_container(&cache.structures)
    {
        if container
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            let amount = creep.store().get_used_capacity(Some(ResourceType::Energy));

            let _ = creep.drop(ResourceType::Energy, Some(amount));

            let priority = scale_haul_priority(
                container.store().get_capacity(Some(ResourceType::Energy)),
                amount,
                HaulingPriority::Energy,
                false
            );

            cache.hauling.create_order(
                creep.try_raw_id().unwrap(),
                Some(ResourceType::Energy),
                Some(creep.store().get_used_capacity(Some(ResourceType::Energy))),
                priority,
                HaulingType::Pickup,
            );
        } else if creep.pos().is_near_to(container.pos()) {
            let _ = creep.transfer(&container, ResourceType::Energy, None);
        } else {
            creep.better_move_to(creep_memory, cache, container.pos(), 1);
        }
    } else {
        let _ = creep.drop(ResourceType::Energy, None);
    }
}

pub fn repair_container(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut CachedRoom) -> bool {

    if creep.store().get_used_capacity(None) == 0 {
        return false;
    }

    let container =
        cache.resources.sources[creep_memory.task_id.unwrap() as usize].get_container(&cache.structures);

    if let Some(container) = container {
        if container.hits() < container.hits_max() {
            if container.pos().get_range_to(creep.pos()) > 1 {
                let _ = creep.say("🚚", false);
                creep.better_move_to(creep_memory, cache, container.pos(), 1);
                return true;
            } else {
                let _ = creep.say("🔧", false);
                let _ = creep.repair(&container);
                return true;
            }
        }
    }
    false
}

pub fn get_aproximate_energy_mined(creep: &Creep, source: &Source) -> u32 {
    let work_parts = creep.body().iter().filter(|x| x.part() == Part::Work && x.hits() > 0).count() as u32;

    let max_mineable = work_parts * 2;
    let source_remaining = source.energy();

    std::cmp::min(max_mineable, source_remaining)
}