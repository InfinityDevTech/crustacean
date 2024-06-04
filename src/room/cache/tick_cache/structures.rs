use std::{cmp, collections::HashMap};

use log::info;
use screeps::{
    find, game, look::{self, LookResult}, ConstructionSite, Creep, HasId, HasPosition, LocalRoomTerrain, ObjectId, OwnedStructureProperties, Part, ResourceType, Room, Ruin, Source, StructureContainer, StructureController, StructureExtension, StructureLink, StructureObject, StructureRoad, StructureSpawn, StructureStorage, StructureTower, Terrain
};

use crate::{memory::ScreepsMemory, room::cache::heap_cache::RoomHeapCache, utils::scale_haul_priority};

use super::hauling::{HaulingCache, HaulingPriority, HaulingType};

#[derive(Debug, Clone)]
pub struct CachedSource {
    pub id: ObjectId<Source>,
    pub creeps: Vec<ObjectId<Creep>>,

    pub link: Option<ObjectId<StructureLink>>,
    pub container: Option<ObjectId<StructureContainer>>,

    pub csites: Vec<ConstructionSite>,
}

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

    pub sources: Vec<CachedSource>,
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

            sources: Vec::new(),
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

        cache.refresh_source_cache(room, heap_cache);
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

                info!("Making offer for non-spawn container {}", priority);

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

    pub fn check_containers(&mut self, hauling: &mut HaulingCache) {
        for container in self.containers.values() {
            let used_capacity = container
                .store()
                .get_used_capacity(Some(ResourceType::Energy));
            let max_capacity = container.store().get_capacity(Some(ResourceType::Energy));

            let mut i = 0;
            let mut is_source_container = false;
            let mut is_controller_container = true;

            loop {
                if self.sources.len() <= i {
                    break;
                }
                if is_source_container {
                    break;
                }

                if let Some(source_container) = self.sources[i].get_container() {
                    if container.id() == source_container.id() {
                        is_source_container = true;
                    }
                }

                i += 1;
            }

            if let Some(controller) = &self.controller.as_ref() {
                if controller.container.is_some() {
                    let controller_container = controller.container.as_ref().unwrap();
                    if container.id() == controller_container.id() {
                        is_controller_container = false;
                    }
                }
            }

            if is_controller_container && used_capacity > 0 {
                if container
                    .pos()
                    .get_range_to(self.spawns.values().next().unwrap().pos())
                    <= 3
                    &&
                    container.store().get_used_capacity(Some(ResourceType::Energy)) as f32 > container.store().get_capacity(Some(ResourceType::Energy)) as f32 * 0.5
                {
                    let priority = scale_haul_priority(
                        container.store().get_capacity(Some(ResourceType::Energy)),
                        container.store().get_used_capacity(Some(ResourceType::Energy)),
                        HaulingPriority::Minerals,
                        true
                    );
                    hauling.create_order(
                        container.raw_id(),
                        Some(ResourceType::Energy),
                        Some(container
                            .store()
                            .get_used_capacity(Some(ResourceType::Energy))),
                        priority,
                        HaulingType::Offer,
                    );

                } else {
                    // Caused an issue where a hauler would grab from the container
                    // Container would go lower than 50 percent, than the hauler
                    // Would stick it back in said container :)
                    let priority = scale_haul_priority(
                        container.store().get_capacity(Some(ResourceType::Energy)),
                        container.store().get_used_capacity(Some(ResourceType::Energy)),
                        HaulingPriority::Energy,
                        true
                    );

                    hauling.create_order(
                        container.raw_id(),
                        Some(ResourceType::Energy),
                        Some(container
                            .store()
                            .get_used_capacity(Some(ResourceType::Energy))),
                        priority,
                        HaulingType::Offer,
                    );
                }
            }

            if !is_source_container && (used_capacity as f32) <= (max_capacity as f32 * 0.5) {
                if container
                    .pos()
                    .get_range_to(self.spawns.values().next().unwrap().pos())
                    <= 3
                {
                    let priority = scale_haul_priority(
                        container.store().get_capacity(Some(ResourceType::Energy)),
                        container.store().get_used_capacity(Some(ResourceType::Energy)),
                        HaulingPriority::Spawning,
                        true
                    );

                    hauling.create_order(
                        container.raw_id(),
                        Some(ResourceType::Energy),
                        Some(container
                            .store()
                            .get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap()),
                        priority,
                        HaulingType::Transfer,
                    );
                } else {
                    let priority = scale_haul_priority(
                        container.store().get_capacity(Some(ResourceType::Energy)),
                        container.store().get_used_capacity(Some(ResourceType::Energy)),
                        HaulingPriority::Energy,
                        true
                    );

                    hauling.create_order(
                        container.raw_id(),
                        Some(ResourceType::Energy),
                        Some(container
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

    pub fn refresh_source_cache(&mut self, room: &Room, cache: &mut RoomHeapCache) {
        // Fetch from heap, if not available, fetch from game.
        // Then push to heap ofc.
        let sources = if cache.sources.is_empty() {
            let sources = room.find(find::SOURCES, None);

            for source in &sources {
                cache.sources.push(source.id());
            }

            sources
        } else {
            let mut vec = vec![];
            for sourceid in &cache.sources {
                vec.push(game::get_object_by_id_typed(sourceid).unwrap());
            }

            vec
        };

        for source in sources {
            let csites = source.pos().find_in_range(find::CONSTRUCTION_SITES, 2);

            let constructed_source = CachedSource {
                id: source.id(),
                creeps: Vec::new(),

                link: None,
                container: None,
                csites,
            };

            self.sources.push(constructed_source);
        }
    }
}

impl CachedSource {
    pub fn get_container(&mut self) -> Option<StructureContainer> {
        if let Some(container_id) = self.container {
            return Some(game::get_object_by_id_typed(&container_id).unwrap());
        }

        let source = game::get_object_by_id_typed(&self.id).unwrap();
        let pos = source.pos();

        let mut find = pos.find_in_range(find::STRUCTURES, 1);
        find.retain(|c| matches!(c, StructureObject::StructureContainer(_)));

        if !find.is_empty() {
            let container = find[0].clone();
            if let StructureObject::StructureContainer(container) = container {
                self.container = Some(container.id());
                return Some(container);
            }
            return None;
        }

        None
    }

    pub fn parts_needed(&self) -> u8 {
        let source: Source = game::get_object_by_id_typed(&self.id).unwrap();
        let max_energy = source.energy_capacity();

        // Each work part equates to 2 energy per tick
        // Each source refills energy every 300 ticks.
        let max_work_needed = (max_energy / 300) + 2;

        let work_parts_needed = max_work_needed - self.calculate_work_parts() as u32;

        cmp::max(work_parts_needed, 0) as u8
    }

    pub fn calculate_mining_spots(&self, room: &Room) -> u8 {
        let source = game::get_object_by_id_typed(&self.id).unwrap();

        let x = source.pos().x().u8();
        let y = source.pos().y().u8();

        let areas = room.look_for_at_area(look::TERRAIN, y - 1, x - 1, y + 1, x + 1);
        let mut available_spots = 0;

        for area in areas {
            match area.look_result {
                LookResult::Terrain(Terrain::Plain) => available_spots += 1,
                LookResult::Terrain(Terrain::Swamp) => available_spots += 1,
                _ => {}
            }
        }

        available_spots
    }

    pub fn calculate_work_parts(&self) -> u8 {
        let creeps = &self.creeps;

        let mut work_parts: u8 = 0;

        for creep in creeps {
            let creep = game::get_object_by_id_typed(creep);
            if creep.is_none() {
                continue;
            }

            let mut body = creep.unwrap().body();
            body.retain(|part| part.part() == Part::Work);

            work_parts += body.len() as u8
        }

        work_parts
    }
}
