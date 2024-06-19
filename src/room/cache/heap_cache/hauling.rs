use std::collections::HashMap;

use js_sys::Object;
use screeps::{game, HasPosition, ObjectId, Position, RawObjectId};

#[derive(Debug, Clone)]
pub struct HeapHaulingReservation {
    pub target_id: RawObjectId,
    pub creeps_assigned: Vec<String>,
    pub amount_reserved: u32,
}

#[derive(Debug, Clone)]
pub struct HeapHaulingCache {
    pub reserved_orders: HashMap<RawObjectId, HeapHaulingReservation>

}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl HeapHaulingCache {
    pub fn new() -> HeapHaulingCache {
        HeapHaulingCache {
            reserved_orders: HashMap::new()
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl HeapHaulingReservation {
    pub fn get_target_position(&self) -> Option<Position> {
        let target = game::get_object_by_id_erased(&self.target_id);

        target.as_ref()?;

        Some(target.unwrap().pos())
    }
}