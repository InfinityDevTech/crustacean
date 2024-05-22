use std::{cmp::min, str::FromStr};

use screeps::{
    find, game, Creep, HasPosition, ObjectId, Resource, ResourceType, Room, RoomName, SharedCreepProperties, Structure, StructureObject, Transferable
};

use crate::{
    memory::{CreepMemory, HaulOrder, ScreepsMemory}, room, traits::creep::CreepExtensions
};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory) {
    let creep_memory = memory.creeps.get(&creep.name()).unwrap();

    let order = creep_memory.task_id;
    if let Some(order_id) = order {
        execute_order(creep, memory, order_id)
    } else {
        let new_order = find_haul_order(creep, memory);
        if let Some(order) = new_order {
            execute_order(creep, memory, order.id);
        }
    }
}

pub fn find_haul_order(creep: &Creep, memory: &mut ScreepsMemory) -> Option<HaulOrder> {
    let creep_memory = memory.creeps.get(&creep.name()).unwrap();
    let room_memory = memory.rooms.get_mut(&RoomName::from_str(&creep_memory.owning_room).unwrap()).unwrap();

    let order_list = room_memory.haul_orders.clone();
    let mut orders = order_list.values().collect::<Vec<&HaulOrder>>();
    orders.sort_by(|a, b| a.priority.cmp(&b.priority));

    let unresponded_orders = orders.into_iter().filter(|&order| order.responder.is_none());

    if let Some(order) = unresponded_orders.into_iter().next() {
        let order = room_memory.haul_orders.get_mut(&order.id).unwrap();
        order.add_responder(creep);
        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
        creep_memory.task_id = Some(order.id);
        Some(order.clone())
    } else {
        None
    }
}

pub fn execute_order(creep: &Creep, memory: &mut ScreepsMemory, order_id: u128) {
    let room_memory = memory.rooms.get_mut(&RoomName::from_str(&memory.creeps.get(&creep.name()).unwrap().owning_room).unwrap()).unwrap();

    let order = room_memory.get_haul_order(order_id).unwrap();
    let pickup_target = order.target_id;

    let position = order.get_target_position();

    if position.get_range_to(creep.pos()) > 1 {
        let creep_memory = memory.creeps.get_mut(&creep.name()).unwrap();
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