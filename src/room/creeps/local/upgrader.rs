use screeps::{Creep, HasPosition, MaybeHasId, OwnedStructureProperties, Part, ResourceType, SharedCreepProperties};

use crate::{
    memory::ScreepsMemory,
    movement::move_target::MoveOptions,
    room::cache::
        {
            hauling::{HaulTaskRequest, HaulingType},
            RoomCache,
        }
    ,
    traits::{
        creep::CreepExtensions, intents_tracking::CreepExtensionsTracking, room::RoomExtensions
    },
    utils::{get_room_sign, under_storage_gate},
};

use super::hauler::execute_order;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_upgrader(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if creep.spawning() || creep.tired() {
        creep.bsay("😴", false);
        return;
    }

    if get_energy(creep, memory, cache) || sign_controller(creep, memory, cache) {
        return;
    }

    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();

    let controller = cached_room.structures.controller.as_ref().unwrap();

    if controller.pos().get_range_to(creep.pos()) > 3 {
        creep.bsay("🚚 CTRL", false);
        creep.better_move_to(
            memory,
            cached_room,
            controller.pos(),
            3,
            MoveOptions::default(),
        );
    } else {
        creep.bsay("⚡", false);
        let _ = creep.upgrade_controller(controller);

        cached_room.stats.energy.spending_upgrading += energy_spent_upgrading(creep);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_energy(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
    let cached_room = cache.rooms.get_mut(&creep_memory.owning_room).unwrap();
    let controller_downgrade = cached_room.structures.controller.as_ref().unwrap().ticks_to_downgrade();

    if creep.room().unwrap().name() != creep_memory.owning_room {
        if let Some(task) = creep_memory.hauling_task.clone() {
            execute_order(creep, memory, cache, &task);
        } else {
            let pos = cached_room.structures.controller.as_ref().unwrap().pos();
            creep.better_move_to(
                memory,
                cache.rooms.get_mut(&creep.room().unwrap().name()).unwrap(),
                pos,
                3,
                MoveOptions::default(),
            );
        }
        return true;
    }

    if (creep.store().get_used_capacity(Some(ResourceType::Energy)) as f32)
        <= (creep.store().get_capacity(Some(ResourceType::Energy)) as f32 * 0.75)
    {
        if let Some(room_storage) = cached_room.structures.storage.as_ref() {
            //let energy_gate = cached_room.storage_status.wanted_energy as f32 * 0.8;

            if under_storage_gate(cached_room, 0.9) && controller_downgrade > Some(5000) {
                if let Some(controller) = cached_room.structures.controller.as_ref() {
                    if creep.pos().get_range_to(controller.pos()) > 3 {
                        creep.better_move_to(
                            memory,
                            cached_room,
                            controller.pos(),
                            3,
                            MoveOptions::default(),
                        );
                        return true;
                    }
                }
                creep.bsay("🚫", false);
                return true;
            }
        }

        if let Some(controller_link) = cached_room.structures.links().controller.as_ref() {
            if controller_link
                .store()
                .get_used_capacity(Some(ResourceType::Energy))
                > 0
            {
                if creep.pos().is_near_to(controller_link.pos()) {
                    let _ = creep.ITwithdraw(controller_link, ResourceType::Energy, None);

                    return false;
                } else {
                    let pos = controller_link.pos();

                    creep.better_move_to(
                        memory,
                        cached_room,
                        pos,
                        1,
                        MoveOptions::default(),
                    );

                    return true;
                }
            }
        }
        let container = &cached_room.structures.containers().controller;
        if let Some(container) = container {
            if container.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
                return false;
            }

            if creep.pos().get_range_to(container.pos()) > 1 {
                let pos = container.pos();

                creep.better_move_to(
                    memory,
                    cached_room,
                    pos,
                    1,
                    MoveOptions::default(),
                );
                return true;
            } else {
                let _ = creep.ITwithdraw(container, ResourceType::Energy, None);

                // This is dumb as hell, I can harvest and transfer in the same tick.
                // But I cant upgrade and withdraw in the same tick.
                return false;
            }
        } else if creep.store().get_used_capacity(None) == 0 {
            let priority = creep.store().get_free_capacity(Some(ResourceType::Energy));

            if cached_room.rcl <= 2 {
                if let Some(task) = creep_memory.hauling_task.clone() {
                    execute_order(creep, memory, cache, &task);

                    return true;
                } else {
                    cached_room.hauling.wanting_orders.push(
                        HaulTaskRequest::default()
                            .creep_name(creep.name())
                            .resource_type(ResourceType::Energy)
                            .haul_type(vec![
                                HaulingType::Offer,
                                HaulingType::Pickup,
                                HaulingType::Withdraw,
                            ])
                            .clone(),
                    );

                    return true;
                }
            }
            cached_room.hauling.create_order(
                creep.try_raw_id().unwrap(),
                None,
                Some(ResourceType::Energy),
                Some(
                    creep
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        .try_into()
                        .unwrap(),
                ),
                priority as f32,
                HaulingType::Transfer,
            );
            return false;
        }
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn sign_controller(creep: &Creep, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> bool {
    let room = creep.room().unwrap();
    let cache = cache.rooms.get_mut(&room.name()).unwrap();

    if cache.structures.controller.is_none() {
        return false;
    }

    if let Some(controller) = cache.structures.controller.as_ref() {
        if controller.owner().is_some() && controller.owner().unwrap().username() == "Player94" {
            if let Some(sign) = controller.sign() {
                if sign.text() == "👀 - Always watching, always waiting." {
                    return false;
                }
            }

            if !creep.pos().is_near_to(controller.pos()) {
                creep.better_move_to(
                    memory,
                    cache,
                    controller.pos(),
                    1,
                    MoveOptions::default(),
                );
            } else {
                let _ = creep.ITsign_controller(controller, "👀 - Always watching, always waiting.");
            }
        }
    }

    // E46N38 is a remote of NeonCamoflauge, and I want to sign it ( Totally to not fuck with him ;) ).
    if !memory.remote_rooms.contains_key(&room.name()) && !memory.rooms.contains_key(&creep.room().unwrap().name()) && room.name() != "E46N38" {
        return false;
    }

    if let Some(controller) = cache.structures.controller.as_ref() {
        if !creep.room().unwrap().is_my_sign() {
            if creep.pos().is_near_to(controller.pos()) {
                if memory
                    .remote_rooms
                    .contains_key(&creep.room().unwrap().name())
                {
                    let _ = creep.ITsign_controller(controller, &get_room_sign(true));
                } else {
                    let _ = creep.ITsign_controller(controller, &get_room_sign(false));
                }
            } else {
                creep.better_move_to(
                    memory,
                    cache,
                    controller.pos(),
                    1,
                    MoveOptions::default(),
                );
            }
            return true;
        }
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn energy_spent_upgrading(creep: &Creep) -> u32 {
    let parts = creep
        .body()
        .iter()
        .filter(|x| x.part() == Part::Work && x.hits() > 0)
        .count() as u32;

    parts * 2
}
