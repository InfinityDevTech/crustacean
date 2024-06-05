use std::collections::HashMap;

use screeps::{
    find, ConstructionSite, HasId, HasPosition, LocalRoomTerrain, ObjectId, OwnedStructureProperties, ResourceType, Room, Ruin, StructureContainer, StructureController, StructureExtension, StructureLink, StructureObject, StructureRoad, StructureSpawn, StructureStorage, StructureTower
};

use crate::{memory::ScreepsMemory, room::cache::heap_cache::RoomHeapCache, utils::scale_haul_priority};

use super::hauling::{HaulingCache, HaulingPriority, HaulingType};

#[derive(Debug, Clone)]
pub struct CachedController {
    pub controller: StructureController,
    pub container: Option<StructureContainer>,
}

#[derive(Debug, Clone)]
pub struct RoomStructureCache {
    pub all_structures: Vec<StructureObject>,
    pub construction_sites: Vec<ConstructionSite>,

    pub needs_repair: Vec<StructureObject>,

    pub ruins: HashMap<ObjectId<Ruin>, Ruin>,
    pub spawns: HashMap<ObjectId<StructureSpawn>, StructureSpawn>,
    pub extensions: HashMap<ObjectId<StructureExtension>, StructureExtension>,
    pub containers: HashMap<ObjectId<StructureContainer>, StructureContainer>,

    pub fast_filler_containers: HashMap<ObjectId<StructureContainer>, StructureContainer>,

    pub controller: Option<CachedController>,
    pub storage: Option<StructureStorage>,

    pub terrain: LocalRoomTerrain,
    pub roads: HashMap<ObjectId<StructureRoad>, StructureRoad>,

    pub links: HashMap<ObjectId<StructureLink>, StructureLink>,
    pub towers: HashMap<ObjectId<StructureTower>, StructureTower>,
}

impl RoomStructureCache {
    pub fn new_from_room(
        room: &Room,
        _memory: &mut ScreepsMemory,
        heap_cache: &mut RoomHeapCache,
    ) -> RoomStructureCache {
        let mut cache = RoomStructureCache {
            all_structures: Vec::new(),
            construction_sites: Vec::new(),
            needs_repair: Vec::new(),

            ruins: HashMap::new(),
            towers: HashMap::new(),
            spawns: HashMap::new(),
            containers: HashMap::new(),
            fast_filler_containers: HashMap::new(),

            controller: None,
            storage: None,

            terrain: LocalRoomTerrain::from(room.get_terrain()),
            roads: HashMap::new(),

            links: HashMap::new(),
            extensions: HashMap::new(),
        };

        if let Some(controller) = room.controller() {
            let containers = controller.pos().find_in_range(find::STRUCTURES, 2);
            let container = containers
                .iter()
                .find(|c| matches!(c, StructureObject::StructureContainer(_)));

            let mut cid = None;

            if let Some(StructureObject::StructureContainer(container)) = container {
                cid = Some(container);
            }

            let cached_controller = CachedController {
                controller,
                container: cid.cloned(),
            };

            cache.controller = Some(cached_controller);
        }

        cache.refresh_structure_cache(room);
        cache.refresh_spawn_cache(room);
        cache.refresh_construction_cache(room);
        cache.refresh_ruin_cache(room);
        cache
    }

    pub fn refresh_spawn_cache(&mut self, room: &Room) {
        let spawns = room.find(find::MY_SPAWNS, None);

        for spawn in spawns {
            self.spawns.insert(spawn.id(), spawn);
        }
    }

    pub fn temp(&mut self, hauling: &mut HaulingCache) {
        for source in self.extensions.values() {
            if source.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                let priority = scale_haul_priority(
                    source.store().get_capacity(Some(ResourceType::Energy)),
                    source.store().get_used_capacity(Some(ResourceType::Energy)),
                    HaulingPriority::Spawning,
                    true
                );

                hauling.create_order(
                    source.raw_id(),
                    Some(ResourceType::Energy),
                    Some(source
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        .try_into()
                        .unwrap()),
                    priority,
                    HaulingType::Transfer,
                );
            }
        }
    }

    pub fn refresh_ruin_cache(&mut self, room: &Room) {
        //if game::time() % 100 != 0 {
        //    return;
        //}

        let ruins = room.find(find::RUINS, None).into_iter();

        for ruin in ruins {
            self.ruins.insert(ruin.id(), ruin);
        }
    }

    pub fn refresh_structure_cache(&mut self, room: &Room) {
        let structures = room.find(find::STRUCTURES, None).into_iter();

        for structure in structures {
            self.all_structures.push(structure.clone());

            if let Some(repairable) = structure.as_repairable() {
                if repairable.hits() < repairable.hits_max() {
                    self.needs_repair.push(structure.clone());
                }
            }

            match structure {
                StructureObject::StructureTower(tower) => {
                    if !tower.my() {
                        continue;
                    }
                    self.towers.insert(tower.id(), tower);
                }
                StructureObject::StructureExtension(extension) => {
                    if !extension.my() {
                        continue;
                    }
                    self.extensions.insert(extension.id(), extension);
                }
                StructureObject::StructureLink(link) => {
                    if !link.my() {
                        continue;
                    }
                    self.links.insert(link.id(), link);
                }
                StructureObject::StructureRoad(road) => {
                    self.roads.insert(road.id(), road);
                }
                StructureObject::StructureContainer(container) => {
                    self.containers.insert(container.id(), container);
                }
                StructureObject::StructureStorage(storage) => {
                    self.storage = Some(storage);
                }
                _ => {}
            }
        }
    }

    pub fn refresh_construction_cache(&mut self, room: &Room) {
        let mut construction_sites = room.find(find::CONSTRUCTION_SITES, None);

        self.construction_sites.append(&mut construction_sites);
    }
}