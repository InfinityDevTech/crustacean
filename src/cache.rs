use std::collections::HashMap;

use screeps::{StructureType, Structure, ObjectId, Creep, ConstructionSite, LocalCostMatrix, Resource, StructureTower};
use serde::{Deserialize, Serialize};

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsCache {
        pub room_specific: HashMap<String, pub struct {
            pub enemy_creeps: Vec<ObjectId<Creep>>,
            pub towers: Vec<ObjectId<StructureTower>>,
            pub structures: HashMap<StructureType, Vec<ObjectId<Structure>>>,
            pub csites: Vec<ObjectId<ConstructionSite>>,
            pub cost_matrix: Option<LocalCostMatrix>,
            pub energy: Vec<ObjectId<Resource>>
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

    pub fn get_room(&mut self, room_name: &str) -> Option<&mut crate::cache::RoomSpecific> {
        self.room_specific.get_mut(room_name)
    }

    pub fn clean_cache(&mut self) {
        self.room_specific.clear();
    }
}
