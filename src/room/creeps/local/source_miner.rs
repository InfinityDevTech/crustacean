use screeps::{
    game, Creep, ErrorCode, HasHits, HasPosition, MaybeHasId, ResourceType, SharedCreepProperties,
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

use super::hauler;

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let CreepMemory {
        task_id,
        needs_energy,
        ..
    } = memory.creeps.get(&creep.name()).unwrap();

    if task_id.is_none() {
        let _ = creep.say("kurt kob", true);
        let _ = creep.suicide();
    }

    let pointer_index = task_id.unwrap() as usize;
    cache.resources.sources[pointer_index]
        .creeps
        .push(creep.try_id().unwrap());
    let scouted_source = &cache.resources.sources[pointer_index];
    let source = game::get_object_by_id_typed(&scouted_source.id).unwrap();

    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if creep.store().get_used_capacity(Some(ResourceType::Energy))
        >= creep.store().get_capacity(Some(ResourceType::Energy))
    {
        creep_memory.needs_energy = None;
        //if !link_deposit(creep, creep_memory, cache) {
        //    drop_deposit(creep, creep_memory, cache);
        //}
    }
    if creep.store().get_free_capacity(None) as f32
        <= (creep.store().get_capacity(None) as f32 * 0.5)
    {
        if !link_deposit(creep, memory.creeps.get_mut(&creep.name()).unwrap(), cache) {
            deposit_energy(creep, memory, cache);
        }

        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            memory.creeps.get_mut(&creep.name()).unwrap().needs_energy = Some(true);
            //harvest_source(creep, source, creep_memory);
        }
    } else {
        harvest_source(creep, source, creep_memory, cache);
    }
}

fn needs_haul_manually(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
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

fn deposit_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if needs_haul_manually(creep, memory, cache) {
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    //if build_around_source(creep, creep_memory, cache) {
    //    return;
    //}

    if repair_container(creep, creep_memory, cache) {
        return;
    }
    let _ = creep.say("📦", false);

    if let Some(container) =
        cache.resources.sources[creep_memory.task_id.unwrap() as usize].get_container()
    {
        if container
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            == 0
        {
            let amount = creep.store().get_used_capacity(Some(ResourceType::Energy));

            let _ = creep.drop(ResourceType::Energy, Some(amount));

            let priority = scale_haul_priority(
                container.store().get_capacity(Some(ResourceType::Energy)) as u32,
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

fn build_around_source(
    creep: &Creep,
    creep_memory: &mut CreepMemory,
    cache: &mut RoomCache,
) -> bool {
    let csites = &cache.resources.sources[creep_memory.task_id.unwrap() as usize].csites;
    if csites.is_empty() {
        return false;
    }

    let csite = csites.first().unwrap();

    if creep.pos().is_near_to(csite.pos()) {
        let _ = creep.say("🔨", false);
        let _ = creep.build(csite);
        true
    } else {
        let _ = creep.say("🚚", false);
        creep.better_move_to(creep_memory, cache, csite.pos(), 1);
        true
    }
}

fn repair_container(creep: &Creep, creep_memory: &mut CreepMemory, cache: &mut RoomCache) -> bool {
    let container =
        cache.resources.sources[creep_memory.task_id.unwrap() as usize].get_container();

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
