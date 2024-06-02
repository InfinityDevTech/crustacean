use screeps::{find, game, Creep, HasId, HasPosition, ObjectId, RawObjectId, ResourceType, SharedCreepProperties, StructureContainer, StructureExtension, StructureObject, StructureProperties, StructureType};

use wasm_bindgen::JsCast;

use crate::{memory::ScreepsMemory, room::cache::tick_cache::RoomCache, traits::creep::CreepExtensions};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();

    if creep.spawning() || creep.tired() {
        let _ = creep.say("😴", false);
        return;
    }

    if creep_memory.fastfiller_container.is_none() {
        if let Some(container_id) = find_container(creep, cache) {
            creep_memory.fastfiller_container = Some(container_id.into());
        } else {
            return;
        }
    }

    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
        let _ = creep.say("WTHD", false);
        let container_id = creep_memory.fastfiller_container.unwrap();
        let container = game::get_object_by_id_typed(&container_id).unwrap();

        if creep.pos().is_near_to(container.pos()) {
            let _ = creep.withdraw(&container, ResourceType::Energy, None);
        } else {
            creep.better_move_to(creep_memory, cache, container.pos(), 1);
        }

        return;
    }

    let possible_targets = find_possible_targets(creep, cache);
    if possible_targets.is_empty() {
        return;
    }

    let target_id = possible_targets[0];
    let target = game::get_object_by_id_erased(&target_id).unwrap();

    if creep.pos().is_near_to(target.pos()) {
        let _ = creep.transfer(target.unchecked_ref::<StructureExtension>(), ResourceType::Energy, None);
    } else {
        creep.better_move_to(creep_memory, cache, target.pos(), 1);
    }

}

pub fn find_possible_targets(creep: &Creep, cache: &RoomCache) -> Vec<RawObjectId> {
    let find_call = creep.pos().find_in_range(find::STRUCTURES, 1);

    let mut possible_targets = Vec::new();

    for target in find_call {
        match target {
            StructureObject::StructureExtension(extension) => {
                if extension.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                    possible_targets.push(extension.raw_id());
                }
            }

            StructureObject::StructureSpawn(spawn) => {
                if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                    possible_targets.push(spawn.raw_id());
                }
            }
            _ => {}
        }
    }

    possible_targets
}

pub fn find_container(creep: &Creep, cache: &RoomCache) -> Option<ObjectId<StructureContainer>> {
    let possible_containers = creep.pos().find_in_range(find::STRUCTURES, 1);

    for container in possible_containers {
        if let StructureObject::StructureContainer(container) = container {
            return Some(container.id());
        }
    }

    None
}