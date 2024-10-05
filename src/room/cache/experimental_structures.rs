use std::str::FromStr;

use js_sys::{Array, JsString};
use screeps::{RoomName, StructureContainer, StructureLink, StructureObject};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js_src/test.js")]
extern "C" {
    // Returns an array 4 long:
    // 0 - All Structures
    // 1 - Repairable
    // 2 - Container
    // 3 - Link
    fn do_classify_find(room_name: JsString) -> Array;
}

pub fn do_find(room_name: &RoomName) -> (Vec<StructureObject>, Vec<StructureObject>, Vec<StructureContainer>, Vec<StructureLink>) {
    let res = do_classify_find(JsString::from_str(room_name.to_string().as_str()).unwrap());

    if res.length() == 0 {
        return (vec![], vec![], vec![], vec![]);
    }

    let all_js: Array = res.get(0).into();
    let repairable_js: Array = res.get(1).into();
    let container: Array = res.get(2).into();
    let links: Array = res.get(3).into();

    let all_rs: Vec<StructureObject> = all_js.iter().map(|structure| structure.into()).collect();
    let repairable_rs: Vec<StructureObject> = repairable_js.iter().map(|structure| structure.into()).collect();
    let container_rs: Vec<StructureContainer> = container.iter().map(|structure| structure.into()).collect();
    let links_rs: Vec<StructureLink> = links.iter().map(|structure| structure.into()).collect();

    (all_rs, repairable_rs, container_rs, links_rs)
}