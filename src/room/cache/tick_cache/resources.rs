use std::{cmp, collections::HashMap};

use screeps::{find, game, look::{self, LookResult}, ConstructionSite, Creep, HasId, HasPosition, Mineral, ObjectId, Part, Resource, ResourceType, Room, Source, StructureContainer, StructureLink, StructureObject, Terrain};

use crate::{memory::ScreepsMemory, room::cache::heap_cache::RoomHeapCache, utils::scale_haul_priority};

use super::{hauling::{HaulingCache, HaulingPriority, HaulingType}, structures::RoomStructureCache, RoomCache};

#[derive(Debug, Clone)]
pub struct CachedSource {
    pub id: ObjectId<Source>,
    pub creeps: Vec<ObjectId<Creep>>,

    pub link: Option<ObjectId<StructureLink>>,
    pub container: Option<ObjectId<StructureContainer>>,

    pub csites: Vec<ConstructionSite>,
}

#[derive(Debug, Clone)]
pub struct RoomResourceCache {
    pub sources: Vec<CachedSource>,
    pub mineral: Option<Mineral>,

    pub dropped_energy: Vec<Resource>,
    pub dropped_resources: HashMap<ResourceType, Vec<Resource>>,
}

impl RoomResourceCache {
    pub fn new_from_room(room: &Room, _memory: &mut ScreepsMemory, heap_cache: &mut RoomHeapCache) -> RoomResourceCache {
        let mut cache = RoomResourceCache {
            sources: Vec::new(),
            mineral: None,

            dropped_energy: Vec::new(),
            dropped_resources: HashMap::new(),
        };

        cache.refresh_resource_cache(room);
        cache.refresh_source_cache(room, heap_cache);
        cache.refresh_minerals(room);
        cache
    }

    pub fn refresh_minerals(&mut self, room: &Room) {
        let minerals = room.find(find::MINERALS, None);

        for mineral in minerals {
            self.mineral = Some(mineral);
        }
    }

    pub fn haul_dropped_resources(&self, hauling: &mut HaulingCache) {
        for resource in &self.dropped_energy {
            let priority = scale_haul_priority(resource.amount(), resource.amount(), HaulingPriority::Energy, false);
            hauling.create_order(resource.id().into(), Some(resource.resource_type()), Some(resource.amount()), priority, HaulingType::Pickup);
        }
    }

    pub fn refresh_resource_cache(&mut self, room: &Room) {
        let dropped_resources = room.find(find::DROPPED_RESOURCES, None);

        for resource in dropped_resources {
            if resource.resource_type() == screeps::ResourceType::Energy {
                self.dropped_energy.push(resource);
            } else if let Some(resource_vec) = self.dropped_resources.get_mut(&resource.resource_type()) {
                resource_vec.push(resource);
            } else {
                self.dropped_resources.insert(resource.resource_type(), vec![resource]);
            }
        }
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
    pub fn get_container(&mut self, structures: &RoomStructureCache) -> Option<StructureContainer> {
        if let Some(container_id) = self.container {
            return Some(game::get_object_by_id_typed(&container_id).unwrap());
        }

        let source = game::get_object_by_id_typed(&self.id).unwrap();
        let pos = source.pos();

        let mut found_container = None;
        for container in structures.containers.values() {
            if container.pos().is_near_to(pos) {
                self.container = Some(container.id());
                found_container = Some(container);

                break;
            }
        }

        if found_container.is_some() {
            self.container = Some(found_container.unwrap().id());
            return Some(found_container.unwrap().clone());
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

pub fn haul_containers(cache: &mut RoomCache) {
    for container in cache.structures.containers.values() {
        let used_capacity = container
            .store()
            .get_used_capacity(Some(ResourceType::Energy));
        let max_capacity = container.store().get_capacity(Some(ResourceType::Energy));

        let mut i = 0;
        let mut is_source_container = false;
        let mut is_controller_container = true;

        loop {
            if cache.resources.sources.len() <= i {
                break;
            }
            if is_source_container {
                break;
            }

            if let Some(source_container) = cache.resources.sources[i].get_container(&cache.structures) {
                if container.id() == source_container.id() {
                    is_source_container = true;
                }
            }

            i += 1;
        }

        if let Some(controller) = &cache.structures.controller.as_ref() {
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
                .get_range_to(cache.structures.spawns.values().next().unwrap().pos())
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
                cache.hauling.create_order(
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

                cache.hauling.create_order(
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
                .get_range_to(cache.structures.spawns.values().next().unwrap().pos())
                <= 3
            {
                let priority = scale_haul_priority(
                    container.store().get_capacity(Some(ResourceType::Energy)),
                    container.store().get_used_capacity(Some(ResourceType::Energy)),
                    HaulingPriority::Spawning,
                    true
                );

                cache.hauling.create_order(
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

                cache.hauling.create_order(
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