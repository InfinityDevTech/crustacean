use std::collections::HashMap;

use screeps::{find, HasId, ObjectId, Resource, Room, Source, StructureController, StructureExtension, StructureLink, StructureObject, StructureSpawn, StructureTower};

#[derive(Debug, Clone)]
pub struct RoomStructureCache {
    pub sources: HashMap<ObjectId<Source>, Source>,
    pub spawns: HashMap<ObjectId<StructureSpawn>, StructureSpawn>,
    pub extensions: HashMap<ObjectId<StructureExtension>, StructureExtension>,
    pub dropped_resources: Vec<ObjectId<Resource>>,

    pub controller: Option<StructureController>,

    pub links: HashMap<ObjectId<StructureLink>, StructureLink>,
    pub towers: HashMap<ObjectId<StructureTower>, StructureTower>
}

impl RoomStructureCache {
    pub fn new_from_room(room: &Room) -> RoomStructureCache {
        let mut cache = RoomStructureCache {
            sources: HashMap::new(),
            towers: HashMap::new(),
            spawns: HashMap::new(),
            dropped_resources: Vec::new(),

            controller: None,

            links: HashMap::new(),
            extensions: HashMap::new()
        };

        if let Some(controller) = room.controller() {
            cache.controller = Some(controller);
        }

        cache.refresh_source_cache(room);
        cache.refresh_structure_cache(room);
        cache.refresh_spawn_cache(room);
        cache.refresh_dropped_resources(room);
        cache
    }

    pub fn refresh_dropped_resources(&mut self, room: &Room) {
        let resources = room.find(find::DROPPED_RESOURCES, None);
        for resource in resources {
            self.dropped_resources.push(resource.id());
        }
    }

    pub fn refresh_spawn_cache(&mut self, room: &Room) {
        let spawns = room.find(find::MY_SPAWNS, None);

        for spawn in spawns {
            self.spawns.insert(spawn.id(), spawn);
        }
    }

    pub fn refresh_structure_cache(&mut self, room: &Room) {
        let structures = room.find(find::MY_STRUCTURES, None).into_iter();

        for structure in structures {
            match structure {
                StructureObject::StructureTower(tower) => {
                    self.towers.insert(tower.id(), tower);
                }
                StructureObject::StructureExtension(extension) => {
                    self.extensions.insert(extension.id(), extension);
                }
                StructureObject::StructureLink(link) => {
                    self.links.insert(link.id(), link);
                }
                _ => {}
            }
        }
    }

    pub fn refresh_source_cache(&mut self, room: &Room) {
        let sources = room.find(find::SOURCES, None);
        for source in sources {
            self.sources.insert(source.id(), source);
        }
    }
}