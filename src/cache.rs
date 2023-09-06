use std::collections::HashMap;

use screeps::{StructureType, Structure, ObjectId, Creep, ConstructionSite, LocalCostMatrix, Resource};
use serde::{Deserialize, Serialize};

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsCache {
        pub room_specific: HashMap<String, pub struct {
            pub enemy_creeps: Vec<ObjectId<Creep>>,
            pub towers: Vec<ObjectId<Structure>>,
            pub structures: HashMap<StructureType, Vec<ObjectId<Structure>>>,
            pub csites: HashMap<String, Vec<ObjectId<ConstructionSite>>>,
            pub cost_matrix: Option<LocalCostMatrix>,
            pub energy: HashMap<String, Vec<ObjectId<Resource>>>
        }>,
    }
}

unsafe impl Send for ScreepsCache {}
unsafe impl Sync for ScreepsCache {}

impl ScreepsCache {
    pub fn init_cache() -> ScreepsCache {
        ScreepsCache {
            room_specific: HashMap::new(),
        }
    }

    pub fn clean_cache(&mut self) {
        self.room_specific.clear();
    }
}
