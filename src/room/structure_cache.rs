use std::{collections::HashMap, hash::Hash};

use screeps::{find, HasId, ObjectId, Room, Source, StructureObject::StructureTower, StructureSpawn, StructureTower as StructType};

#[derive(Debug, Clone)]
pub struct RoomStructureCache {
    pub sources: HashMap<ObjectId<Source>, Source>,
    pub spawns: HashMap<ObjectId<StructureSpawn>, StructureSpawn>,
    pub towers: HashMap<ObjectId<StructType>, StructType>
}

impl RoomStructureCache {
    pub fn new_from_room(room: &Room) -> RoomStructureCache {
        let mut cache = RoomStructureCache {
            sources: HashMap::new(),
            towers: HashMap::new(),
            spawns: HashMap::new()
        };

        cache.refresh_source_cache(room);
        cache.refresh_tower_cache(room);
        cache.refresh_spawn_cache(room);
        cache
    }

    pub fn refresh_spawn_cache(&mut self, room: &Room) {
        let spawns = room.find(find::MY_SPAWNS, None);

        for spawn in spawns {
            self.spawns.insert(spawn.id(), spawn);
        }
    }

    pub fn refresh_tower_cache(&mut self, room: &Room) {
        let towers = room.find(find::MY_STRUCTURES, None).into_iter().filter_map(|structure| {
            match structure {
                StructureTower(tower) => {
                    return Some(tower);
                }
                _ => None
            }
        });

        for tower in towers {
            self.towers.insert(tower.id(), tower);
        }
    }

    pub fn refresh_source_cache(&mut self, room: &Room) {
        let sources = room.find(find::SOURCES, None);
        for source in sources {
            self.sources.insert(source.id(), source);
        }
    }
}