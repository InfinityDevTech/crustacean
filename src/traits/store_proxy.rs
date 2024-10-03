use std::{collections::HashMap, sync::Mutex};

use log::info;
use screeps::{game, RawObjectId, ResourceType, StructureStorage};
use wasm_bindgen::JsCast;

lazy_static::lazy_static! {
    pub static ref NONE_STORED_CAPACITIES: Mutex<HashMap<RawObjectId, u32>> = Mutex::new(HashMap::new());
    pub static ref SOME_STORED_CAPACITIES: Mutex<HashMap<RawObjectId, HashMap<ResourceType, u32>>> = Mutex::new(HashMap::new());
    pub static ref STORED_FREE_CAPACITIES: Mutex<HashMap<RawObjectId, HashMap<ResourceType, u32>>> = Mutex::new(HashMap::new());
    pub static ref STORED_USED_CAPACITIES: Mutex<HashMap<RawObjectId, HashMap<ResourceType, u32>>> = Mutex::new(HashMap::new());
}

pub fn get_capacity(obj: RawObjectId, ty: Option<ResourceType>) -> u32 {
    if ty.is_some() {
        let mut lock = SOME_STORED_CAPACITIES.lock().unwrap();
        let ty = ty.unwrap();

        if let Some(cached_obj) = lock.get_mut(&obj) {
            if let Some(cached_amt) = cached_obj.get(&ty) {
                info!("Cached.");
                *cached_amt
            } else {
                let game_obj = game::get_object_by_id_erased(&obj).unwrap();
                let store = game_obj.unchecked_ref::<StructureStorage>().store();
                let amt = store.get_capacity(None);

                cached_obj.insert(ty, amt);

                info!("Had to fetch.");

                amt
            }
        } else {
            lock.insert(obj, HashMap::new());
            drop(lock);

            get_capacity(obj, Some(ty))
        }
    } else {
        let mut lock = NONE_STORED_CAPACITIES.lock().unwrap();
        if let Some(cached_obj) = lock.get(&obj) {
            *cached_obj
        } else {
            let game_obj = game::get_object_by_id_erased(&obj).unwrap();
            let store = game_obj.unchecked_ref::<StructureStorage>().store();
            let amt = store.get_capacity(None);

            lock.insert(obj, amt);

            amt
        }
    }
}