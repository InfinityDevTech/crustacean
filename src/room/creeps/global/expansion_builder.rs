use screeps::{Creep, HasPosition, Position, ResourceType, RoomCoordinate, SharedCreepProperties};

use crate::{
    memory::{Role, ScreepsMemory},
    movement::move_target::MoveOptions,
    room::{cache::tick_cache::{hauling::{HaulTaskRequest, HaulingType}, resources::CachedSource, RoomCache},
    creeps::local::hauler::execute_order},
    traits::{creep::CreepExtensions, intents_tracking::CreepExtensionsTracking},
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_expansionbuilder(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let room_cache = cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap();

    if creep.spawning() {
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name());
    if creep_memory.is_none() || creep.spawning() {
        return;
    }
    let creep_memory = creep_memory.unwrap();

    if creep_memory.target_room.is_none() {
        creep_memory.role = Role::Recycler;
        return;
    }

    let target_room = creep_memory.target_room.unwrap();

    let cloned_csites = room_cache
    .structures
    .construction_sites
    .clone();
    let mut non_road_csite_count = cloned_csites
        .iter()
        .filter(|s| s.structure_type() != screeps::StructureType::Road).clone();

    if let Some(haul_task) = &creep_memory.hauling_task.clone() {
        execute_order(creep, memory, cache, haul_task);

        return;
    }

    let meet_position = Position::new(
        unsafe { RoomCoordinate::unchecked_new(25) },
        unsafe { RoomCoordinate::unchecked_new(25) },
        target_room,
    );

    if creep.pos().get_range_to(meet_position) < 24 {
        let room_cache = cache.rooms.get_mut(&target_room).unwrap();
        let needs_energy = creep_memory.needs_energy.unwrap_or(false);

        if needs_energy {
            if room_cache.creeps.creeps_of_role.get(&Role::Harvester).unwrap_or(&Vec::new()).len() < 2 {
                let mut target = room_cache
                .resources
                .sources
                .iter()
                .filter(|s| s.source.energy() > 0)
                .collect::<Vec<&CachedSource>>();

            target.sort_by_key(|s| s.source.pos().get_range_to(creep.pos()));

            if let Some(target) = target.first() {
                if creep.pos().is_near_to(target.source.pos()) {
                    let _ = creep.ITharvest(&target.source);
                } else {
                    creep.better_move_to(
                        memory,
                        room_cache,
                        target.source.pos(),
                        1,
                        MoveOptions::default(),
                    );
                }
            }
            } else if let Some(task) = &creep_memory.hauling_task.clone() {
                execute_order(creep, memory, cache, &task);

                return;
            } else {
                room_cache.hauling.wanting_orders.push(HaulTaskRequest::default().creep_name(creep.name()).resource_type(ResourceType::Energy).haul_type(vec![HaulingType::Pickup, HaulingType::Offer, HaulingType::Withdraw]).clone());
            }

            let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
            if creep.store().get_free_capacity(None) == 0 {
                creep_memory.needs_energy = Some(false);
            }
        } else if non_road_csite_count.clone().count() >= 1 {
            if let Some(spawn) = room_cache.structures.spawns.values().next() {
                if spawn.store().get_free_capacity(None) > 0 {
                    if creep.pos().is_near_to(spawn.pos()) {
                        let _ = creep.ITtransfer(spawn, ResourceType::Energy, None);

                        return;
                    } else {
                        creep.better_move_to(
                            memory,
                            room_cache,
                            spawn.pos(),
                            1,
                            MoveOptions::default(),
                        );

                        return;
                    }
                }
            }

            if let Some(csite) = non_road_csite_count.next() {
                if creep.pos().get_range_to(csite.pos()) <= 3 {
                    let _ = creep.ITbuild(csite);
                } else {
                    creep.better_move_to(
                        memory,
                        room_cache,
                        csite.pos(),
                        3,
                        MoveOptions::default(),
                    );
                }
            }
        } else {
            if let Some(spawn) = room_cache.structures.spawns.values().next() {
                if spawn.store().get_free_capacity(None) > 0 {
                    if creep.pos().is_near_to(spawn.pos()) {
                        let _ = creep.ITtransfer(spawn, ResourceType::Energy, None);
                    } else {
                        creep.better_move_to(
                            memory,
                            room_cache,
                            spawn.pos(),
                            1,
                            MoveOptions::default(),
                        );
                    }
                }
            }

            let controller = &room_cache.structures.controller.as_ref().unwrap();

            if creep.pos().get_range_to(controller.controller.pos()) <= 3 {
                let _ = creep.upgrade_controller(&controller.controller);
            } else {
                creep.better_move_to(
                    memory,
                    room_cache,
                    controller.controller.pos(),
                    1,
                    MoveOptions::default(),
                );
            }
        }

        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
        if creep.store().get_used_capacity(None) == 0 {
            creep_memory.needs_energy = Some(true);
        }
    } else {
        creep.better_move_to(
            memory,
            room_cache,
            meet_position,
            22,
            MoveOptions::default()
                .avoid_enemies(true)
                .avoid_hostile_rooms(true),
        );
    }
}