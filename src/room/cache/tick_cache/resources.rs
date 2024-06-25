use std::{cmp, collections::HashMap};

use screeps::{find, game, look::{self, LookResult}, ConstructionSite, Creep, HasId, HasPosition, Mineral, ObjectId, Part, Resource, ResourceType, Room, Source, StructureContainer, StructureLink, StructureProperties, Terrain};

use crate::{memory::{Role, ScreepsMemory}, room::cache::heap_cache::RoomHeapCache, utils::scale_haul_priority};

use super::{hauling::{HaulingPriority, HaulingType}, structures::RoomStructureCache, CachedRoom, RoomCache};

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

    pub total_energy: u32,
    pub dropped_energy_amount: u32,
    pub energy_in_storing_structures: u32,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomResourceCache {
    pub fn new_from_room(room: &Room, _memory: &mut ScreepsMemory, heap_cache: &mut RoomHeapCache) -> RoomResourceCache {
        let mut cache = RoomResourceCache {
            sources: Vec::new(),
            mineral: None,

            total_energy: 0,
            dropped_energy_amount: 0,
            energy_in_storing_structures: 0,

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

    pub fn refresh_resource_cache(&mut self, room: &Room) {
        let dropped_resources = room.find(find::DROPPED_RESOURCES, None);

        for resource in dropped_resources {
            if resource.resource_type() == screeps::ResourceType::Energy {
                self.total_energy += resource.amount();
                self.dropped_energy_amount += resource.amount();
                
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CachedSource {
    pub fn get_container(&mut self, structures: &RoomStructureCache) -> Option<StructureContainer> {
        if let Some(container_id) = self.container {
            return Some(game::get_object_by_id_typed(&container_id).unwrap());
        }

        let source = game::get_object_by_id_typed(&self.id).unwrap();
        let pos = source.pos();

        let mut found_container = None;

        if let Some(containers) = &structures.containers.source_container {
            for container in containers {
                if container.pos().is_near_to(pos) {
                    found_container = Some(container);
                    break;
                }
            }
        }

        if found_container.is_some() {
            self.container = Some(found_container.unwrap().id());
            return Some(found_container.unwrap().clone());
        }

        None
    }

    pub fn get_link(&mut self, structures: &RoomStructureCache) -> Option<StructureLink> {
        if let Some(link_id) = self.link {
            return Some(game::get_object_by_id_typed(&link_id).unwrap());
        }

        let source = game::get_object_by_id_typed(&self.id).unwrap();
        let pos = source.pos();

        let mut found_link = None;

        if let Some(links) = &structures.links.source {
            for link in links {
                if link.pos().is_near_to(pos) {
                    found_link = Some(link);
                    break;
                }
            }
        }

        if found_link.is_some() {
            self.link = Some(found_link.unwrap().id());
            return Some(found_link.unwrap().clone());
        }

        None
    }

    pub fn parts_needed(&self) -> u8 {
        let source: Source = game::get_object_by_id_typed(&self.id).unwrap();
        let max_energy = source.energy_capacity();

        // Each work part equates to 2 energy per tick
        // Each source refills energy every 300 ticks.
        let max_work_needed = (max_energy / 600) + 1;
        let current_work = self.calculate_work_parts();

        // Fixes issue where if we spawn with more parts,
        // We would integer underflow and return u32::MAX parts.
        if current_work as u32 >= max_work_needed {
            //info!("Dodging underflow bug in parts_needed");
            return 0;
        }

        let work_parts_needed = max_work_needed - current_work as u32;

        cmp::max(work_parts_needed, 6) as u8
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

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_remotes(launching_room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    for remote_name in memory.rooms.get(&launching_room.name()).unwrap().remotes.clone().iter() {
        let remote_room = game::rooms().get(*remote_name);

        // If we have no visibility, continue...
        if remote_room.is_none() {
            continue;
        }
        let remote_room = remote_room.unwrap();

        cache.create_if_not_exists(&remote_room, memory, Some(remote_room.name()));

        let cached_room = cache.rooms.get_mut(remote_name).unwrap().clone();
        let owning_room = cache.rooms.get_mut(&launching_room.name()).unwrap();

        for resource in &cached_room.resources.dropped_energy {
            let priority = scale_haul_priority(20000, resource.amount(), HaulingPriority::Energy, false);
            owning_room.resources.dropped_energy_amount += resource.amount();
            owning_room.hauling.create_order(resource.id().into(), None, Some(resource.resource_type()), Some(resource.amount()), -priority, HaulingType::Pickup);
        }

        if cached_room.structures.containers.source_container.is_none() {
            continue;
        }

        for container in &cached_room.structures.containers.source_container.unwrap() {
            if container.store().get_used_capacity(None) > 0 {
                let priority = scale_haul_priority(
                    container.store().get_free_capacity(None) as u32,
                    container.store().get_used_capacity(None),
                    HaulingPriority::Energy,
                    true
                );

                owning_room.resources.total_energy += container.store().get_used_capacity(None);
                owning_room.resources.energy_in_storing_structures += container.store().get_used_capacity(None);

                owning_room.hauling.create_order(container.id().into(), Some(container.structure_type()), Some(ResourceType::Energy), Some(container.store().get_used_capacity(Some(ResourceType::Energy))), priority, HaulingType::Offer);
            } else if container.store().get_free_capacity(None) == 0 {
                owning_room.resources.total_energy += container.store().get_used_capacity(None);
                owning_room.resources.energy_in_storing_structures += container.store().get_used_capacity(None);

                owning_room.hauling.create_order(container.id().into(), Some(container.structure_type()), Some(ResourceType::Energy), Some(container.store().get_used_capacity(Some(ResourceType::Energy))), 0.0, HaulingType::Offer)
            }
        }
    }
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_containers(cached_room: &mut CachedRoom) {
    if let Some(controller_container) = &cached_room.structures.containers.controller {
        if controller_container.store().get_used_capacity(None) < (controller_container.store().get_capacity(None) / 2) && cached_room.structures.links.controller.is_none() {
            let mut priority = scale_haul_priority(
                controller_container.store().get_free_capacity(None) as u32,
                controller_container.store().get_used_capacity(None),
                HaulingPriority::Upgrading,
                false
            );

            if cached_room.structures.links.controller.is_none() {
                priority -= 20.0;
            }

            let priority = priority - controller_container.pos().get_range_to(cached_room.structures.spawns.values().next().unwrap().pos()) as f32;

            cached_room.hauling.create_order(controller_container.id().into(), Some(controller_container.structure_type()), Some(ResourceType::Energy), Some(controller_container.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap()), priority as f32, HaulingType::Transfer);
        }
    }

    if let Some(fastfiller_containers) = &cached_room.structures.containers.fast_filler {
        if cached_room.creeps.creeps_of_role.get(&Role::BaseHauler).unwrap_or(&Vec::new()).is_empty() {

        for fastfiller_container in fastfiller_containers {
            if fastfiller_container.store().get_free_capacity(None) > 0 {
                let priority = scale_haul_priority(
                    fastfiller_container.store().get_free_capacity(None) as u32,
                    fastfiller_container.store().get_used_capacity(None),
                    HaulingPriority::FastFillerContainer,
                    false
                );

                cached_room.hauling.create_order(fastfiller_container.id().into(), Some(fastfiller_container.structure_type()), Some(ResourceType::Energy), Some(fastfiller_container.store().get_free_capacity(Some(ResourceType::Energy)).try_into().unwrap()), priority, HaulingType::Transfer);
            }
        }
    }
    }

    for source in &mut cached_room.resources.sources {
        let container = source.get_container(&cached_room.structures);

        if container.is_none() {
            continue;
        }

        let container = container.unwrap();

        if container.store().get_used_capacity(None) > 0 {
            let mut priority = scale_haul_priority(
                container.store().get_free_capacity(None) as u32,
                container.store().get_used_capacity(None),
                HaulingPriority::Energy,
                true
            );

            // NEVER, and i mean NEVER leave containers full.
            if container.store().get_free_capacity(None) == 0 {
                priority -= 1000000.0;
            }

            cached_room.hauling.create_order(container.id().into(), Some(container.structure_type()), Some(ResourceType::Energy), Some(container.store().get_used_capacity(Some(ResourceType::Energy))), priority, HaulingType::Offer);
        } else if container.store().get_free_capacity(None) == 0 {
            cached_room.hauling.create_order(container.id().into(), Some(container.structure_type()), Some(ResourceType::Energy), Some(container.store().get_used_capacity(Some(ResourceType::Energy))), 0.0, HaulingType::Offer)
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn haul_dropped_resources(cached_room: &mut CachedRoom) {
    for resource in &cached_room.resources.dropped_energy {
        let priority = scale_haul_priority(50000, resource.amount(), HaulingPriority::Energy, false);
        cached_room.hauling.create_order(resource.id().into(), None, Some(resource.resource_type()), Some(resource.amount()), -priority, HaulingType::Pickup);
    }
}