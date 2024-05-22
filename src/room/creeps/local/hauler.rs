use std::{cmp::min, str::FromStr};

use screeps::{
    find, game, Creep, HasPosition, ObjectId, Resource, ResourceType, RoomName, SharedCreepProperties, Structure, StructureObject, Transferable
};

use crate::{
    memory::{CreepMemory, ScreepsMemory}, room, traits::creep::CreepExtensions
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory) {
    let creep_memory = memory.get_creep_mut(&creep.name());

    let order = creep_memory.t_id;
    if let Some(order_id) = order {
        execute_order(creep, memory, order_id)
    } else {
        let room_memory = memory.get_room_mut(&RoomName::from_str(&creep_memory.o_r).unwrap());
        let new_order = room_memory.find_haul_order(creep, memory);
        if let Some(order) = new_order {
            execute_order(creep, memory, order.id);
        }
    }
}

pub fn execute_order(creep: &Creep, memory: &mut ScreepsMemory, order_id: u128) {
    let room_memory = memory.get_room_mut(&RoomName::from_str(&memory.get_creep(&creep.name()).o_r).unwrap());

    let order = room_memory.get_haul_order(order_id).unwrap();
    let pickup_target = order.target_id;

    let position = order.get_target_position();

    if position.get_range_to(creep.pos()) > 1 {
        let creep_memory = memory.get_creep_mut(&creep.name());
        creep.better_move_to(creep_memory, position, 1);
        return;
    }

    match order.haul_type {
        crate::room::hauling::HaulType::Pickup => {
            let target: Option<Resource> = game::get_object_by_id_typed(&ObjectId::from(pickup_target));

            if target.is_none() {
                room_memory.destroy_haul_order(order_id);
                return;
            }

            let _ = creep.pickup(&target.unwrap());
        },
        crate::room::hauling::HaulType::Deposit => {
            //let target = game::get_object_by_id_erased(&pickup_target);

            //if target.is_none() {
            //    room_memory.destroy_haul_order(order_id);
            //    return;
            //}

            //let transferrable: &dyn Transferable = target.unwrap().as_ref();

            //let _ = creep.transfer(target.unwrap().as_ref(), order.target_type, Some(min(creep.store().get_used_capacity(Some(order.target_type)), 32)));
        },
        crate::room::hauling::HaulType::Withdraw => todo!(),
    }
}