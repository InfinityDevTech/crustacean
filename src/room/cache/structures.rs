use std::collections::HashMap;

use screeps::{
    find, ConstructionSite, HasId, HasPosition, LocalRoomTerrain, ObjectId, OwnedStructureProperties, ResourceType, Room, Ruin, StructureContainer, StructureController, StructureExtension, StructureExtractor, StructureFactory, StructureInvaderCore, StructureLab, StructureLink, StructureNuker, StructureObject, StructureObserver, StructurePowerSpawn, StructureProperties, StructureRampart, StructureRoad, StructureSpawn, StructureStorage, StructureTerminal, StructureTower, StructureType, Tombstone
};

use crate::{constants::NO_RCL_PLACEABLES, heap_cache::heap_room::HeapRoom, memory::ScreepsMemory};

use super::resources::RoomResourceCache;

#[derive(Debug, Clone)]
pub struct CachedRoomContainers {
    pub controller: Option<StructureContainer>,
    pub fast_filler: Option<Vec<StructureContainer>>,
    pub source_container: Option<Vec<StructureContainer>>,
    pub mineral: Option<StructureContainer>,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CachedRoomContainers {
    pub fn new() -> Self {
        CachedRoomContainers {
            controller: None,
            fast_filler: None,
            source_container: None,
            mineral: None,
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
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
    room: Room,
    pub needs_repair: Vec<StructureObject>,

    pub ramparts: Vec<StructureRampart>,

    pub ruins: HashMap<ObjectId<Ruin>, Ruin>,
    pub spawns: HashMap<ObjectId<StructureSpawn>, StructureSpawn>,
    pub extensions: HashMap<ObjectId<StructureExtension>, StructureExtension>,
    pub containers: HashMap<ObjectId<StructureContainer>, StructureContainer>,
    pub links: HashMap<ObjectId<StructureLink>, StructureLink>,

    pub invader_core: Option<StructureInvaderCore>,
    pub controller: Option<StructureController>,
    pub storage: Option<StructureStorage>,
    pub observer: Option<StructureObserver>,
    pub nuker: Option<StructureNuker>,
    pub terminal: Option<StructureTerminal>,
    pub factory: Option<StructureFactory>,
    pub power_spawn: Option<StructurePowerSpawn>,
    pub extractor: Option<StructureExtractor>,
    pub labs: HashMap<ObjectId<StructureLab>, StructureLab>,

    pub terrain: LocalRoomTerrain,
    pub roads: HashMap<ObjectId<StructureRoad>, StructureRoad>,

    pub towers: HashMap<ObjectId<StructureTower>, StructureTower>,

    construction_sites: Option<Vec<ConstructionSite>>,
    tombstones: Option<HashMap<ObjectId<Tombstone>, Tombstone>>,
    classified_links: Option<CachedRoomLinks>,
    classified_containers: Option<CachedRoomContainers>,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomStructureCache {
    pub fn new_from_room(
        room: &Room,
        resource_cache: &mut RoomResourceCache,
        memory: &mut ScreepsMemory,
        _heap_cache: &mut HeapRoom,
    ) -> RoomStructureCache {
        let room_memory = memory.rooms.get_mut(&room.name());

        let mut cache = RoomStructureCache {
            room: room.clone(),
            needs_repair: Vec::new(),

            ramparts: Vec::new(),

            ruins: HashMap::new(),
            towers: HashMap::new(),
            spawns: HashMap::new(),
            links: HashMap::new(),
            containers: HashMap::new(),

            invader_core: None,
            controller: None,
            storage: None,
            observer: None,
            nuker: None,
            terminal: None,
            factory: None,
            power_spawn: None,
            extractor: None,
            labs: HashMap::new(),

            terrain: LocalRoomTerrain::from(room.get_terrain()),
            roads: HashMap::new(),


            extensions: HashMap::new(),
            tombstones: None,
            classified_containers: Some(CachedRoomContainers::new()),
            classified_links: Some(CachedRoomLinks::new()),
            construction_sites: None,
        };

        if let Some(controller) = room.controller() {
            if let Some(room_memory) = room_memory {
                room_memory.rcl = controller.level();

                if room_memory.max_rcl < controller.level() {
                    room_memory.max_rcl = controller.level();
                }
            }

            cache.controller = Some(controller);
        }

        cache.refresh_structure_cache(resource_cache, memory);

        cache
    }

    pub fn links(&self) -> &CachedRoomLinks {
        return self.classified_links.as_ref().unwrap();
    }

    pub fn containers(&self) -> &CachedRoomContainers {
        return self.classified_containers.as_ref().unwrap();
    }

    // This is all to avoid a clone.
    // Plus, this makes it lazy.
    pub fn all_structures(&self) -> Vec<StructureObject> {
        let mut vec = Vec::new();

        // Ramparts
        vec.extend(self.ramparts.iter().map(|rampart| StructureObject::from(rampart.clone())));
        // Spawns
        vec.extend(self.spawns.values().map(|spawn| StructureObject::from(spawn.clone())));
        // Extensions
        vec.extend(self.extensions.values().map(|extension| StructureObject::from(extension.clone())));
        // Containers
        vec.extend(self.containers.values().map(|container| StructureObject::from(container.clone())));
        // Links
        vec.extend(self.links.values().map(|link| StructureObject::from(link.clone())));
        // Labs
        vec.extend(self.labs.values().map(|lab| StructureObject::from(lab.clone())));
        // Invader Core
        if let Some(invader_core) = &self.invader_core { vec.push(StructureObject::from(invader_core.clone())); }
        // Controller
        if let Some(controller) = &self.controller { vec.push(StructureObject::from(controller.clone())); }
        // Storage
        if let Some(storage) = &self.storage { vec.push(StructureObject::from(storage.clone())); }
        // Observer
        if let Some(observer) = &self.observer { vec.push(StructureObject::from(observer.clone())); }
        // Nuker
        if let Some(nuker) = &self.nuker { vec.push(StructureObject::from(nuker.clone())); }
        // Terminal
        if let Some(terminal) = &self.terminal { vec.push(StructureObject::from(terminal.clone())); }
        // Factory
        if let Some(factory) = &self.factory { vec.push(StructureObject::from(factory.clone())); }
        // Power Spawn
        if let Some(power_spawn) = &self.power_spawn { vec.push(StructureObject::from(power_spawn.clone())); }
        // Extractor
        if let Some(extractor) = &self.extractor { vec.push(StructureObject::from(extractor.clone())); }
        // Roads
        vec.extend(self.roads.values().map(|road| StructureObject::from(road.clone())));

        // Towers
        vec.extend(self.towers.values().map(|tower| StructureObject::from(tower.clone())));

        vec
    }

    fn run_structure_find(&mut self) -> Vec<StructureObject> {
        self.room.find(find::STRUCTURES, None)
    }

    fn repairables(&mut self) {
        for structure in self.all_structures().into_iter() {
            if let Some(damagable) = structure.as_attackable() {
                if damagable.hits() < damagable.hits_max() {
                    self.needs_repair.push(structure);
                }
            }
        }
    }

    fn skip_check(&mut self, can_be_placed: bool, structure: &StructureObject) -> bool {
        if !can_be_placed && !NO_RCL_PLACEABLES.contains(&structure.structure_type()) {
            return true;
        } else if !can_be_placed {
            if let StructureObject::StructureContainer(container) = structure {
                self.containers.insert(container.id(), container.clone());
                return true;
            }
        }

        false
    }

    fn classify_structure(&mut self, resource_cache: &mut RoomResourceCache, structure: StructureObject, has_links: &mut bool, has_containers: &mut bool) {
        match structure {
            StructureObject::StructureTower(tower) => {
                self.towers.insert(tower.id(), tower);
            }
            StructureObject::StructureExtension(extension) => {
                self.extensions.insert(extension.id(), extension);
            }
            StructureObject::StructureLink(link) => {
                resource_cache.energy_in_storing_structures +=
                    link.store().get_used_capacity(Some(ResourceType::Energy));

                *has_links = true;
                self.links.insert(link.id(), link);
            }
            StructureObject::StructureRoad(road) => {
                self.roads.insert(road.id(), road);
            }
            StructureObject::StructureInvaderCore(core) => {
                self.invader_core = Some(core);
            }
            StructureObject::StructureContainer(container) => {
                resource_cache.energy_in_storing_structures += container
                    .store()
                    .get_used_capacity(Some(ResourceType::Energy));

                *has_containers = true;
                self.containers.insert(container.id(), container);
            }
            StructureObject::StructureRampart(rampart) => {
                self.ramparts.push(rampart);
            }
            StructureObject::StructureStorage(storage) => {
                resource_cache.energy_in_storing_structures += storage
                    .store()
                    .get_used_capacity(Some(ResourceType::Energy));

                self.storage = Some(storage);
            }
            StructureObject::StructureSpawn(spawn) => {
                self.spawns.insert(spawn.id(), spawn);
            }
            StructureObject::StructureNuker(nuker) => {
                self.nuker = Some(nuker);
            }
            StructureObject::StructureObserver(observer) => {
                self.observer = Some(observer);
            }
            StructureObject::StructureTerminal(terminal) => {
                self.terminal = Some(terminal);
            }
            StructureObject::StructureFactory(factory) => {
                self.factory = Some(factory);
            }
            StructureObject::StructurePowerSpawn(power_spawn) => {
                self.power_spawn = Some(power_spawn);
            }
            StructureObject::StructureLab(lab) => {
                self.labs.insert(lab.id(), lab);
            }
            StructureObject::StructureExtractor(extractor) => {
                self.extractor = Some(extractor);
            }
            _ => {}
        }
    }

    pub fn refresh_structure_cache(
        &mut self,
        resource_cache: &mut RoomResourceCache,
        memory: &mut ScreepsMemory
    ) {
        let room_memory = memory.rooms.get_mut(&self.room.name());

        /*
        let mut can_structures_be_placed = true;
        if let Some(controller) = self.room.controller() {
            if !controller.my() {
                can_structures_be_placed = false;
            }
        } else {
            can_structures_be_placed = false;
        }*/

        let mut check_ownable = false;
        if let Some(room_memory) = room_memory {
            if room_memory.rcl < room_memory.max_rcl {
                check_ownable = true;
            }
        }

        let mut has_containers = false;
        let mut has_links = false;

        for structure in self.run_structure_find().into_iter() {
            //if self.skip_check(can_structures_be_placed, &structure) {
            //    continue;
            //}

            // Dont to the is_active check UNLESS we downgraded.
            // Its very expensive from what I have heard.
            // This information has been reported by: Gadjung
            if check_ownable && !structure.is_active() {
                continue;
            }

            // TODO: Improve this code...
            if let Some(ownable) = structure.as_owned() {
                if !ownable.my() && structure.structure_type() != StructureType::InvaderCore {
                    continue;
                }
            }

            self.classify_structure(resource_cache, structure, &mut has_links, &mut has_containers);
        }

        self.repairables();

        if has_containers {
            self.process_containers(resource_cache);
        }

        if has_links {
            self.process_links(resource_cache);
        }
    }

    pub fn process_links(&mut self, resource_cache: &mut RoomResourceCache) {
        //if self.classified_links.is_some() {
        //    return &self.classified_links.as_ref().unwrap();
        //}

        let mut controller = None;
        let mut fast_filler = None;
        let mut storage = None;
        let mut link_sources = Vec::new();

        // TODO:
        // Do I use a find call?
        // I mean, its only called once, and I think passing the resource cache would get fucky.
        //let sources = self.room.find(find::SOURCES, None);
        //let room_heap = &heap().rooms.lock().unwrap();
        //let sources = &room_heap.get(&self.room.name()).unwrap().sources;

        //let sources = sources.iter().map(|s| game::get_object_by_id_typed(s).unwrap()).collect::<Vec<Source>>();

        for link in self.links.values() {
            if let Some(room_controller) = &self.controller {
                if link.pos().in_range_to(room_controller.pos(), 3) {
                    controller = Some(link.clone());
                }
            }

            if let Some(spawn) = self.spawns.values().next() {
                if link.pos().in_range_to(spawn.pos(), 1) {
                    fast_filler = Some(link.clone())
                }
            }

            if let Some(sstorage) = &self.storage {
                if link.pos().in_range_to(sstorage.pos(), 2) {
                    storage = Some(link.clone());
                }
            }

            for source in resource_cache.sources.iter_mut() {
                if link.pos().in_range_to(source.source.pos(), 2) {
                    source.link = Some(link.clone());
                    link_sources.push(link.clone());
                }
            }
        }

        let link_sources = if link_sources.is_empty() { None } else { Some(link_sources) };

        let classified = CachedRoomLinks {
            controller,
            fast_filler,
            source: link_sources,
            storage,
        };

        self.classified_links = Some(classified);
    }

    pub fn process_containers(&mut self, resource_cache: &mut RoomResourceCache) {
        //if self.classified_containers.is_some() {
        //    return &self.classified_containers.as_ref().unwrap();
        //}

        let mut controller = None;
        let mut mineral_container = None;
        let mut fast_filler = Vec::new();
        let mut source_container = Vec::new();

        // TODO:
        // Do I use a find call?
        // I mean, its only called once, and I think passing the resource cache would get fucky.
        //let sources = self.room.find(find::SOURCES, None);
        //let room_heap = &heap().rooms.lock().unwrap();
        //let sources = &room_heap.get(&self.room.name()).unwrap().sources;

        //let sources = sources.iter().map(|s| game::get_object_by_id_typed(s).unwrap()).collect::<Vec<Source>>();

        for container in self.containers.values() {
            if let Some(room_controller) = &self.controller {
                if container.pos().in_range_to(room_controller.pos(), 2) {
                    controller = Some(container.clone());
                }
            }

            if let Some(spawn) = self.spawns.values().next() {
                if container.pos().in_range_to(spawn.pos(), 2) {
                    fast_filler.push(container.clone());
                }
            }

            if let Some(cmineral) = &resource_cache.mineral {
                if container.pos().in_range_to(cmineral.pos(), 2) {
                    mineral_container = Some(container.clone());
                }
            }

            for source in resource_cache.sources.iter_mut() {
                if container.pos().in_range_to(source.source.pos(), 2) {
                    source.container = Some(container.clone());
                    source_container.push(container.clone());
                }
            }
        }

        let fast_filler = if fast_filler.is_empty() { None } else { Some(fast_filler) };
        let source_container = if source_container.is_empty() { None } else { Some(source_container) };

        let classified = CachedRoomContainers {
            controller,
            fast_filler,
            source_container,
            mineral: mineral_container,
        };

        self.classified_containers = Some(classified);
    }

    pub fn ruins(&mut self) -> &HashMap<ObjectId<Ruin>, Ruin> {
        if !self.ruins.is_empty() {
            return &self.ruins;
        }

        let ruins = self.room.find(find::RUINS, None).into_iter();
        for ruin in ruins {
            self.ruins.insert(ruin.id(), ruin);
        }

        &self.ruins
    }

    pub fn tombstones(&mut self) -> &HashMap<ObjectId<Tombstone>, Tombstone> {
        if self.tombstones.is_some() {
            return self.tombstones.as_ref().unwrap();
        }

        let found_tombstones = self.room.find(find::TOMBSTONES, None).into_iter();
        let mut tombstones = HashMap::new();

        for tombstone in found_tombstones {
            tombstones.insert(tombstone.id(), tombstone);
        }

        self.tombstones = Some(tombstones);

        return self.tombstones.as_ref().unwrap();
    }

    pub fn get_spawns(&self) -> (Vec<StructureSpawn>, Vec<StructureSpawn>) {
        let mut available_spawns = Vec::new();
        let mut unavailable_spawns = Vec::new();

        for spawn in self.spawns.values() {
            if spawn.spawning().is_none() {
                available_spawns.push(spawn.clone())
            } else {
                unavailable_spawns.push(spawn.clone())
            }
        }

        (available_spawns, unavailable_spawns)
    }

    pub fn construction_sites(&mut self) -> &Vec<ConstructionSite> {
        if self.construction_sites.is_some() {
            return self.construction_sites.as_ref().unwrap();
        }
        self.construction_sites = Some(self.room.find(find::CONSTRUCTION_SITES, None));

        return self.construction_sites.as_ref().unwrap();
    }
}
