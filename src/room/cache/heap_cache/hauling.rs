use std::collections::HashMap;

use js_sys::Object;
use screeps::{game, ObjectId, RawObjectId};

#[derive(Debug, Clone)]
pub struct HeapHaulingCache {
    pub reserved_orders: HashMap<RawObjectId, String>
}

impl HeapHaulingCache {
    pub fn new() -> HeapHaulingCache {
        HeapHaulingCache {
            reserved_orders: HashMap::new()
        }
    }
}