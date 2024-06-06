use screeps::{
    game, Creep, HasHits, HasPosition, MaybeHasId, ResourceType, SharedCreepProperties,
    Source,
};

use crate::{
    memory::{CreepMemory, Role, ScreepsMemory},
    room::cache::tick_cache::{
        hauling::{HaulingPriority, HaulingType},
        RoomCache,
    },
    traits::creep::CreepExtensions, utils::scale_haul_priority,
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name());

    if creep_memory.is_none() || creep_memory.as_ref().unwrap().task_id.is_none() {
        let _ = creep.say("kurt kob", true);
        let _ = creep.suicide();
        return;
    }

    let creep_memory = creep_memory.unwrap();

    let pointer_index = creep_memory.task_id.unwrap() as usize;
    let scouted_source = &mut cache.resources.sources[pointer_index];
    scouted_source.creeps.push(creep.try_id().unwrap());

    let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }


    if creep.store().get_free_capacity(None) as f32
        <= (creep.store().get_capacity(None) as f32 * 0.5)
    {
        if !link_deposit(creep, creep_memory, cache) {
            deposit_energy(creep, creep_memory, cache);
        }
        harvest_source(creep, source, creep_memory, cache);
    } else {
        harvest_source(creep, source, creep_memory, cache);
    }
}

fn needs_haul_manually(creep: &Creep, cache: &mut RoomCache) -> bool {
    let count = cache
        .creeps
        .creeps_of_role
        .get(&Role::Hauler)
        .unwrap_or(&vec![])
        .len();

    if count == 0 {
        let _ = creep.drop(ResourceType::Energy, None);

        return false;
    }
    false
}

fn harvest_source(creep: &Creep, source: Source, memory: &mut CreepMemory, cache: &mut RoomCache) {
    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
        return;
    }

    if !creep.pos().is_near_to(source.pos()) {
        let _ = creep.say("🚚 🔋", false);
        creep.better_move_to(memory, cache, source.pos(), 1);
    } else {
        let _ = creep.say("⛏️", false);
        let _ = creep.harvest(&source);
    }
}

fn link_deposit(creep: &Creep, creep_memory: &mut CreepMemory, cache: &RoomCache) -> bool {
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

fn deposit_energy(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut RoomCache) {
    if needs_haul_manually(creep, cache) {
        return;
    }

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

fn repair_container(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut RoomCache) -> bool {
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
