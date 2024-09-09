use core::f32;
use std::collections::HashMap;

use screeps::{
    find, game, look::{self, LookResult}, ConstructionSite, RoomName, Creep, HasId, HasPosition, MapTextStyle, MapVisual, MaybeHasId, Mineral, ObjectId, Part, Position, Resource, ResourceType, Room, RoomCoordinate, RoomXY, SharedCreepProperties, Source, StructureContainer, StructureLink, StructureProperties, StructureType, Terrain
};

#[cfg(feature = "season1")]
use screeps::resource::ResourceType::Score;
#[cfg(feature = "season1")]
use screeps::ScoreContainer;

use crate::{
    heap_cache::heap_room::HeapRoom,
    memory::{Role, ScreepsMemory},
    traits::position::PositionExtensions,
    utils::{self, scale_haul_priority},
};

use super::{
    hauling::{HaulingPriority, HaulingType},
    CachedRoom, RoomCache,
};

#[derive(Debug, Clone)]
pub struct CachedSource {
    pub source: Source,
    pub creeps: Vec<ObjectId<Creep>>,
    pub max_work_parts: u8,
    work_part_count: u8,

    pub link: Option<StructureLink>,
    pub container: Option<StructureContainer>,

    pub csites: Vec<ConstructionSite>,

    lowest_ttl: u32,
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

    #[cfg(feature = "season1")]
    pub season1_score: Vec<ScoreContainer>,
}

impl RoomResourceCache {
    pub fn new_from_room(
        room: &Room,
        _memory: &mut ScreepsMemory,
        heap_cache: &mut HeapRoom,
    ) -> RoomResourceCache {
        let mut cache = RoomResourceCache {
            sources: Vec::new(),
            mineral: None,

            total_energy: 0,
            dropped_energy_amount: 0,
            energy_in_storing_structures: 0,

            dropped_energy: Vec::new(),
            dropped_resources: HashMap::new(),

            #[cfg(feature = "season1")]
            season1_score: Vec::new(),
        };

        cache.refresh_resource_cache(room);
        cache.refresh_source_cache(room, heap_cache);
        cache.refresh_minerals(room);

        #[cfg(feature = "season1")]
        {
            cache.refresh_score_cache(room);
        }

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
            } else if let Some(resource_vec) =
                self.dropped_resources.get_mut(&resource.resource_type())
            {
                resource_vec.push(resource);
            } else {
                self.dropped_resources
                    .insert(resource.resource_type(), vec![resource]);
            }
        }
    }

    pub fn refresh_source_cache(&mut self, room: &Room, cache: &mut HeapRoom) {
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
            let max_parts = utils::source_max_parts(&source);

            let constructed_source = CachedSource {
                source,
                creeps: Vec::new(),
                max_work_parts: max_parts,
                work_part_count: 0,

                link: None,
                container: None,
                csites,

                lowest_ttl: u32::MAX,
            };

            self.sources.push(constructed_source);
        }
    }

    #[cfg(feature = "season1")]
    pub fn refresh_score_cache(&mut self, room: &Room) {
        let score_resources = room.find(find::SCORE_CONTAINERS, None);

        self.season1_score = score_resources;
    }
}

impl CachedSource {
    pub fn get_best_pos_to_stand(
        &mut self,
        creep_positions: &HashMap<RoomXY, Creep>,
    ) -> Option<Position> {
        let available_positions = self.source.pos().get_accessible_positions_around(1);

        if let Some(container) = self.container.as_ref() {
            if available_positions.contains(&container.pos())
                && !creep_positions.contains_key(&container.pos().xy())
            {
                return Some(container.pos());
            }
        }

        available_positions
            .into_iter()
            .find(|&pos| !creep_positions.contains_key(&pos.xy()))
    }

    pub fn parts_needed(&self, cache: &CachedRoom) -> u8 {
        let max_energy = self.source.energy_capacity();

        // Each work part equates to 2 energy per tick
        // Each source refills energy every 300 ticks.
        let max_work_needed = (max_energy / 600) + 1;
        let current_work = self.calculate_work_parts(cache);

        // Fixes issue where if we spawn with more parts,
        // We would integer underflow and return u32::MAX parts.
        if current_work as u32 >= max_work_needed {
            //info!("Dodging underflow bug in parts_needed");
            return 0;
        }

        let work_parts_needed = max_work_needed - current_work as u32;

        work_parts_needed.clamp(0, u8::MAX as u32) as u8
    }

    pub fn can_replace_creep(&self, dist: Position, room: &Room) -> bool {
        let range_to_source = self.source.pos().get_range_to(dist);
        let max_parts = self.max_work_parts;
        let spawn_time = 3 * max_parts as u32;
        let lowest_ttl = self.lowest_ttl;

        if (range_to_source + spawn_time > lowest_ttl) && self.work_part_count <= max_parts {
            if self.creeps.len() > self.calculate_mining_spots(room).into() {
                return false;
            }
            return true;
        }

        false
    }

    pub fn calculate_mining_spots(&self, room: &Room) -> u8 {
        let x = self.source.pos().x().u8();
        let y = self.source.pos().y().u8();

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

    pub fn add_creep(&mut self, creep: &Creep) {
        self.work_part_count += creep
            .body()
            .iter()
            .filter(|p| p.part() == Part::Work)
            .count() as u8;
        self.creeps.push(creep.try_id().unwrap());

        if let Some(ttl) = creep.ticks_to_live() {
            if ttl < self.lowest_ttl {
                self.lowest_ttl = ttl;
            }
        }
    }

    pub fn calculate_work_parts(&self, _cache: &CachedRoom) -> u8 {
        let work_parts: u8 = self.work_part_count;

        /*
        if work_parts > 6 {
            let kreeps = self.creeps.clone();

            let mut smallest = None;
            let mut smallest_score = u32::MAX;
            for creep in kreeps {
                if let Some(game_creep) = game::creeps().get(creep.to_string()) {
                    let score = game_creep.body().iter().filter(|p| p.part() == Part::Work).count() as u32;

                    if score < smallest_score {
                        smallest = Some(game_creep);
                        smallest_score = score;
                    }
                }
            }

            if let Some(creep) = smallest {
                if creep.spawning() {
                    let (spawning, _not_spawning) = cache.structures.get_spawns();

                    for spawn in spawning {
                        if spawn.spawning().is_none() {
                            continue;
                        }

                        if spawn.spawning().unwrap().name() == creep.name() {
                            spawn.spawning().unwrap().cancel();
                        }
                    }
                } else {
                    creep.ITsuicide();
                }
            }
        }
        */

        work_parts
    }
}

pub fn haul_remotes(launching_room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    for remote_name in memory
        .rooms
        .get(&launching_room.name())
        .unwrap()
        .remotes
        .clone()
        .iter()
    {
        let remote_room = game::rooms().get(*remote_name);

        if let Some(remote_room_memory) = memory.remote_rooms.get_mut(remote_name) {
            if remote_room_memory.under_attack {
                let x = unsafe { RoomCoordinate::unchecked_new(46) };
                let y = unsafe { RoomCoordinate::unchecked_new(4) };

                let pos = Position::new(x, y, *remote_name);
                let style = MapTextStyle::default()
                    .font_size(6.0)
                    .align(screeps::TextAlign::Center);
                MapVisual::text(pos, "⚠️".to_string(), style);
                continue;
            }
        }

        // If we have no visibility, continue...
        if remote_room.is_none() {
            continue;
        }
        let remote_room = remote_room.unwrap();

        cache.create_if_not_exists(&remote_room, memory, Some(remote_room.name()));

        let cached_room = cache.rooms.get_mut(remote_name).unwrap().clone();
        let owning_room = cache.rooms.get_mut(&launching_room.name()).unwrap();

        for resource in &cached_room.resources.dropped_energy {
            let amount = resource.amount();

            owning_room.stats.energy.dropped += amount;
            owning_room.hauling.create_order(
                resource.id().into(),
                None,
                Some(resource.resource_type()),
                Some(resource.amount()),
                -(amount as f32),
                HaulingType::Pickup,
            );
        }

        if cached_room
            .structures
            .containers()
            .source_container
            .is_none()
        {
            continue;
        }

        for container in cached_room
            .structures
            .containers()
            .source_container
            .as_ref()
            .unwrap()
        {
            owning_room.resources.total_energy += container.store().get_used_capacity(None);
            owning_room.resources.energy_in_storing_structures +=
                container.store().get_used_capacity(None);

            owning_room.hauling.create_order(
                container.id().into(),
                Some(container.structure_type()),
                Some(ResourceType::Energy),
                Some(
                    container
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy)),
                ),
                -(container
                    .store()
                    .get_used_capacity(Some(ResourceType::Energy)) as f32),
                HaulingType::Offer,
            );
        }
    }
}

pub fn haul_containers(cached_room: &mut CachedRoom) {
    if let Some(controller_container) = &cached_room.structures.containers().controller {
        let upgrader_count = cached_room.creeps.creeps_of_role(Role::Upgrader);

        if !utils::under_storage_gate(cached_room, 1.0) {
            if utils::contains_other_than(&controller_container.store(), ResourceType::Energy) {
                let hashed_store = utils::store_to_hashmap(&controller_container.store());

                for (resource, amount) in hashed_store.iter() {
                    if *resource != ResourceType::Energy {
                        cached_room.hauling.create_order(
                            controller_container.id().into(),
                            Some(controller_container.structure_type()),
                            Some(*resource),
                            Some(*amount),
                            f32::MIN,
                            HaulingType::NoDistanceCalcWithdraw,
                        );
                    }
                }
            }

            if (controller_container.store().get_used_capacity(None)
                < (controller_container.store().get_capacity(None) / 2)
                && cached_room.structures.links().controller.is_none())
                && upgrader_count > 0
            {
                let basehauler_count = cached_room.creeps.creeps_of_role(Role::BaseHauler);

                // TODO: Fix this, its sucking up energy
                // My rooms are dying lmao.
                let mut priority = 21.0;

                if cached_room.structures.links().controller.is_none() {
                    priority -= 20.0;
                }

                if cached_room.rcl <= 3 {
                    priority += 5.0;
                }

                if cached_room.structures.storage.is_some() && basehauler_count == 0 {
                    priority += 10000.0;
                }

                if cached_room.structures.storage.is_none() && cached_room.rcl >= 4 {
                    priority += 500.0;
                }

                priority += controller_container
                    .store()
                    .get_used_capacity(Some(ResourceType::Energy))
                    as f32
                    / 100.0;

                cached_room.stats.energy.in_containers =
                    controller_container.store().get_used_capacity(None);

                cached_room.hauling.create_order(
                    controller_container.id().into(),
                    Some(controller_container.structure_type()),
                    Some(ResourceType::Energy),
                    Some(
                        controller_container
                            .store()
                            .get_free_capacity(Some(ResourceType::Energy))
                            .try_into()
                            .unwrap(),
                    ),
                    priority,
                    HaulingType::NoDistanceCalcTransfer,
                );
            }
        }
    }

    if let Some(fastfiller_containers) = &cached_room.structures.containers().fast_filler {
        for fastfiller_container in fastfiller_containers {
            if utils::contains_other_than(&fastfiller_container.store(), ResourceType::Energy) {
                let hashed_store = utils::store_to_hashmap(&fastfiller_container.store());

                for (resource, amount) in hashed_store.iter() {
                    if *resource != ResourceType::Energy {
                        cached_room.hauling.create_order(
                            fastfiller_container.id().into(),
                            Some(fastfiller_container.structure_type()),
                            Some(*resource),
                            Some(*amount),
                            f32::MIN,
                            HaulingType::NoDistanceCalcWithdraw,
                        );
                    }
                }
            }

            if fastfiller_container.store().get_free_capacity(None) > 0 {
                let priority = scale_haul_priority(
                    fastfiller_container.store().get_free_capacity(None) as u32,
                    fastfiller_container.store().get_used_capacity(None),
                    HaulingPriority::FastFillerContainer,
                    false,
                );

                cached_room.stats.energy.in_containers =
                    fastfiller_container.store().get_used_capacity(None);

                cached_room.hauling.create_order(
                    fastfiller_container.id().into(),
                    Some(fastfiller_container.structure_type()),
                    Some(ResourceType::Energy),
                    Some(
                        fastfiller_container
                            .store()
                            .get_free_capacity(Some(ResourceType::Energy))
                            .try_into()
                            .unwrap(),
                    ),
                    priority,
                    HaulingType::NoDistanceCalcTransfer,
                );
            }
        }
    }

    if let Some(mineral_container) = &cached_room.structures.containers().mineral {
        if mineral_container.store().get_used_capacity(None) > 0 {
            if let Some(mineral) = &cached_room.resources.mineral {
                if utils::contains_other_than(&mineral_container.store(), mineral.mineral_type()) {
                    let hashed_store = utils::store_to_hashmap(&mineral_container.store());

                    for (resource, amount) in hashed_store.iter() {
                        if *resource != ResourceType::Energy {
                            cached_room.hauling.create_order(
                                mineral_container.id().into(),
                                Some(mineral_container.structure_type()),
                                Some(*resource),
                                Some(*amount),
                                f32::MIN,
                                HaulingType::NoDistanceCalcWithdraw,
                            );
                        }
                    }
                }

                let amount = mineral_container
                    .store()
                    .get_used_capacity(Some(mineral.mineral_type()));
                cached_room.hauling.create_order(
                    mineral_container.raw_id(),
                    Some(StructureType::Container),
                    Some(mineral.mineral_type()),
                    Some(amount),
                    -(amount as f32),
                    HaulingType::NoDistanceCalcWithdraw,
                );
            }
        }
    }

    for source in &mut cached_room.resources.sources {
        let container = &source.container.as_ref();

        if container.is_none() {
            continue;
        }

        let container = container.unwrap();

        if container.store().get_used_capacity(None) == 0 {
            continue;
        }

        if utils::contains_other_than(&container.store(), ResourceType::Energy) {
            let hashed_store = utils::store_to_hashmap(&container.store());

            for (resource, amount) in hashed_store.iter() {
                if *resource != ResourceType::Energy {
                    cached_room.hauling.create_order(
                        container.id().into(),
                        Some(container.structure_type()),
                        Some(*resource),
                        Some(*amount),
                        f32::MIN,
                        HaulingType::NoDistanceCalcWithdraw,
                    );
                }
            }
        }

        cached_room.stats.energy.in_containers = container.store().get_used_capacity(None);

        let mut priority = container.store().get_used_capacity(None) as f32;

        if priority >= 2000.0 {
            priority += 99999.0;
        }

        cached_room.hauling.create_order(
            container.id().into(),
            Some(container.structure_type()),
            Some(ResourceType::Energy),
            Some(
                container
                    .store()
                    .get_used_capacity(Some(ResourceType::Energy)),
            ),
            -priority,
            HaulingType::NoDistanceCalcOffer,
        );
    }
}

pub fn haul_dropped_resources(cached_room: &mut CachedRoom) {
    for resource in &cached_room.resources.dropped_energy {
        let amount = resource.amount();

        let mut priority = -(amount as f32);

        if let Some(storage) = &cached_room.structures.storage {
            if storage
                .store()
                .get_used_capacity(Some(ResourceType::Energy))
                < 1000
            {
                priority -= 999999999999.0;
            }
        }

        cached_room.stats.energy.dropped += amount;
        cached_room.hauling.create_order(
            resource.id().into(),
            None,
            Some(resource.resource_type()),
            Some(resource.amount()),
            priority,
            HaulingType::NoDistanceCalcPickup,
        );
    }
}

#[cfg(feature = "season1")]
pub fn haul_score_resources(room_name: &RoomName, cache: &mut RoomCache, memory: &mut ScreepsMemory) {
    use crate::utils::under_storage_gate;

    let responsible_room = utils::find_closest_owned_room(room_name, cache, Some(3));

    if let Some(responsible_room) = responsible_room {
        let my_score_resource = cache.rooms.get(room_name).unwrap().resources.season1_score.clone();

        cache.create_if_not_exists(&game::rooms().get(responsible_room).unwrap(), memory, None);

        let responsible_cache = cache.rooms.get_mut(&responsible_room).unwrap();

        if let Some(storage) = &responsible_cache.structures.storage {
            if storage.store().get_used_capacity(Some(ResourceType::Score)) > 250_000 {
                return;
            }

            for resource in my_score_resource {
                let amt = resource.store().get_used_capacity(Some(ResourceType::Score));
                let prio = if under_storage_gate(responsible_cache, 1.0) {
                    amt as f32
                } else {
                    f32::MIN
                };

                responsible_cache.hauling.create_order(resource.raw_id(), Some(StructureType::Container), Some(ResourceType::Score), Some(amt), prio, HaulingType::Offer);
            }
        }
    }
}