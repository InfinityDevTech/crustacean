use std::collections::HashMap;

use screeps::{StructureType, Structure, ObjectId, Creep, ConstructionSite, LocalCostMatrix, Resource};
use serde::{Deserialize, Serialize};

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsCache {
            pub enemy_creeps: Vec<ObjectId<Creep>>,
            pub towers: Vec<String>,
            pub structures: HashMap<StructureType, Vec<ObjectId<Structure>>>,
            pub csites: HashMap<String, Vec<ObjectId<ConstructionSite>>>,
            pub cost_matrixes: HashMap<String, LocalCostMatrix>,
            pub energy: HashMap<String, Vec<ObjectId<Resource>>>
    }
}

unsafe impl Send for ScreepsCache {}
unsafe impl Sync for ScreepsCache {}

impl ScreepsCache {
    pub fn init_cache() -> ScreepsCache {
        ScreepsCache {
            enemy_creeps: Vec::new(),
            towers: Vec::new(),
            structures: HashMap::new(),
            csites: HashMap::new(),
            cost_matrixes: HashMap::new(),
            energy: HashMap::new(),
        }
    }

    pub fn clean_cache(&mut self) {
        self.structures.clear();
        self.csites.clear();
        self.enemy_creeps.clear();
        self.cost_matrixes.clear();
        self.energy.clear();
    }
}
