use std::collections::HashMap;

use screeps::{
    find, game, ConstructionSite, HasId, HasPosition, LocalRoomTerrain, ObjectId, OwnedStructureProperties, ResourceType, Room, RoomXY, Ruin, StructureContainer, StructureController, StructureExtension, StructureLink, StructureObject, StructureObserver, StructureProperties, StructureRoad, StructureSpawn, StructureStorage, StructureTower, StructureType, Tombstone
};

use crate::{memory::ScreepsMemory, room::cache::heap_cache::RoomHeapCache, utils::get_rampart_repair_rcl};

use super::resources::RoomResourceCache;

#[derive(Debug, Clone)]
pub struct CachedController {
    pub controller: StructureController,
    pub container: Option<StructureContainer>,
}

#[derive(Debug, Clone)]
pub struct CachedRoomContainers {
    pub controller: Option<StructureContainer>,
    pub fast_filler: Option<Vec<StructureContainer>>,
    pub source_container: Option<Vec<StructureContainer>>
}

impl CachedRoomContainers {
    pub fn new() -> Self {
        CachedRoomContainers {
            controller: None,
            fast_filler: None,
            source_container: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedRoomLinks {
    pub controller: Option<StructureLink>,
    pub fast_filler: Option<StructureLink>,
    pub source: Option<Vec<StructureLink>>,
    pub storage: Option<StructureLink>,
}

impl CachedRoomLinks {
    pub fn new() -> Self {
        CachedRoomLinks {
            controller: None,
            fast_filler: None,
            source: None,
            storage: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoomStructureCache {
    pub all_structures: Vec<StructureObject>,
    pub construction_sites: Vec<ConstructionSite>,

    pub needs_repair: Vec<StructureObject>,

    pub ruins: HashMap<ObjectId<Ruin>, Ruin>,
    pub tombstones: HashMap<ObjectId<Tombstone>, Tombstone>,
    pub spawns: HashMap<ObjectId<StructureSpawn>, StructureSpawn>,
    pub extensions: HashMap<ObjectId<StructureExtension>, StructureExtension>,
    pub containers: CachedRoomContainers,
    pub links: CachedRoomLinks,

    pub controller: Option<CachedController>,
    pub storage: Option<StructureStorage>,
    pub observer: Option<StructureObserver>,

    pub terrain: LocalRoomTerrain,
    pub roads: HashMap<ObjectId<StructureRoad>, StructureRoad>,

    pub towers: HashMap<ObjectId<StructureTower>, StructureTower>,
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomStructureCache {
    pub fn new_from_room(
        room: &Room,
        resource_cache: &mut RoomResourceCache,
        _memory: &mut ScreepsMemory,
        heap_cache: &mut RoomHeapCache,
    ) -> RoomStructureCache {
        let mut cache = RoomStructureCache {
            all_structures: Vec::new(),
            construction_sites: Vec::new(),
            needs_repair: Vec::new(),

            ruins: HashMap::new(),
            tombstones: HashMap::new(),
            towers: HashMap::new(),
            spawns: HashMap::new(),
            containers: CachedRoomContainers::new(),
            links: CachedRoomLinks::new(),

            controller: None,
            storage: None,
            observer: None,

            terrain: LocalRoomTerrain::from(room.get_terrain()),
            roads: HashMap::new(),

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

        cache.refresh_construction_cache(room);
        cache.refresh_ruin_cache(room);
        cache.refresh_structure_cache(resource_cache, room);
        cache
    }

    pub fn refresh_ruin_cache(&mut self, room: &Room) {
        let ruins = room.find(find::RUINS, None).into_iter();

        for ruin in ruins {
            self.ruins.insert(ruin.id(), ruin);
        }
    }

    pub fn refresh_structure_cache(&mut self, resource_cache: &mut RoomResourceCache, room: &Room) {
        let structures = room.find(find::STRUCTURES, None).into_iter();

        let mut my_containers = Vec::new();
        let mut my_links = Vec::new();

        for structure in structures {
            self.all_structures.push(structure.clone());

            if let Some(repairable) = structure.as_repairable() {
                let max = if structure.structure_type() == StructureType::Rampart {
                    let controller = self.controller.as_ref().unwrap().controller.clone();
                    get_rampart_repair_rcl(controller.level())
                } else {
                    repairable.hits_max()
                };

                if repairable.hits() < max {
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
                    resource_cache.energy_in_storing_structures += link.store().get_used_capacity(Some(ResourceType::Energy));

                    my_links.push(link);
                }
                StructureObject::StructureRoad(road) => {
                    self.roads.insert(road.id(), road);
                }
                StructureObject::StructureContainer(container) => {
                    resource_cache.energy_in_storing_structures += container.store().get_used_capacity(Some(ResourceType::Energy));

                    my_containers.push(container);
                }
                StructureObject::StructureStorage(storage) => {
                    resource_cache.energy_in_storing_structures += storage.store().get_used_capacity(Some(ResourceType::Energy));

                    self.storage = Some(storage);
                }
                StructureObject::StructureSpawn(spawn) => {
                    if spawn.my() {
                        self.spawns.insert(spawn.id(), spawn);
                    }
                }
                StructureObject::StructureObserver(observer) => {
                    self.observer = Some(observer);
                }
                _ => {}
            }
        }

        for container in my_containers {
            if let Some(controller) = &self.controller {
                if container.pos().get_range_to(controller.controller.pos()) <= 2 {
                    self.containers.controller = Some(container);
                    continue;
                }
            }

            if let Some(spawn) = self.spawns.values().next() {
                if container.pos().get_range_to(spawn.pos()) <= 3 {
                    let fast_filler = self.containers.fast_filler.get_or_insert_with(Vec::new);
                    fast_filler.push(container);
                    continue;
                }
            }

            let found_source_containers = resource_cache.sources.iter().filter_map(|source| {
                if container.pos().get_range_to(game::get_object_by_id_typed(&source.id).unwrap().pos()) <= 2 {
                    Some(container.clone())
                } else {
                    None
                }
            });

            let source_containers = self.containers.source_container.get_or_insert_with(Vec::new);
            source_containers.extend(found_source_containers);
        }

        for link in my_links {
            if let Some(controller) = &self.controller {
                if link.pos().get_range_to(controller.controller.pos()) <= 2 {
                    self.links.controller = Some(link);
                    continue;
                }
            }

            if let Some(spawn) = self.spawns.values().next() {
                let xy = unsafe { RoomXY::unchecked_new(spawn.pos().x().u8(), spawn.pos().y().u8() - 1) };
                if link.pos().xy() == xy {
                    self.links.fast_filler = Some(link);
                    continue;
                }
            }

            if let Some(storage) = &self.storage {
                if link.pos().get_range_to(storage.pos()) <= 2 {
                    self.links.storage = Some(link);
                    continue;
                }
            }

            let found_source_containers = resource_cache.sources.iter().filter_map(|source| {
                if link.pos().get_range_to(game::get_object_by_id_typed(&source.id).unwrap().pos()) <= 2 {
                    Some(link.clone())
                } else {
                    None
                }
            });

            let source_containers = self.links.source.get_or_insert_with(Vec::new);
            source_containers.extend(found_source_containers);
        }

        let tombstones = room.find(find::TOMBSTONES, None);

        for tombstone in tombstones {
            self.tombstones.insert(tombstone.id(), tombstone);
        }
    }

    pub fn refresh_construction_cache(&mut self, room: &Room) {
        let mut construction_sites = room.find(find::CONSTRUCTION_SITES, None);

        self.construction_sites.append(&mut construction_sites);
    }
}