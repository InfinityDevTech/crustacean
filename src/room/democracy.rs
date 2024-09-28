use std::mem;

use log::info;
use screeps::{
    game, look::{self, LookResult}, pathfinder::MultiRoomCostResult, HasPosition, LocalCostMatrix, MapTextStyle, MapVisual, Position, Room, RoomCoordinate, RoomName, RoomPosition, StructureObject, StructureProperties, StructureRoad, StructureType, Terrain
};

use crate::{
    combat::{hate_handler, rank_room, safemode::should_safemode},
    compression::decode_pos_list,
    config::{self, REMOTE_SCAN_FOR_RCL},
    heap,
    memory::{Role, ScreepsMemory},
    room::{
        cache::{hauling, resources, RoomCache},
        creeps::{organizer, recovery::recover_creeps},
        planning::room::{
            plan_room, remotes, roads::{self, plan_main_room_roads}, skippy_base::run_planner,
        },
        tower,
        visuals::run_full_visuals,
    },
    traits::{intents_tracking::RoomExtensionsTracking, room::RoomExtensions},
    utils::{self, distance_transform, new_xy},
};

use super::{
    links,
    planning::{
        self,
        room::{construction::{
            get_containers, get_rcl_2_plan, get_rcl_3_plan, get_rcl_4_plan, get_rcl_5_plan,
            get_rcl_6_plan, get_rcl_7_plan, get_rcl_8_plan, get_roads_and_ramparts,
            plan_remote_containers,
        }, roads::get_all_cached_road_positions},
    },
    visuals::visualise_room_visual,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]

// TODO:
// Separate logic of the room types, eg
// Owned - Remote - Unknown - Enemy - (Potentially, Ally?)
// Change the data stored for scouted rooms, such as safemode, MAX rcl
// Potentially attempt to estimate where the spawn battery is by:
//  Averaging the positions of all the spawns, get the base center
pub fn start_government(room: Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if game::cpu::bucket() < 500 && !room.my() {
        return;
    }

    let starting_cpu = game::cpu::get_used();
    let starting_creep_cpu = cache.creep_cpu.clone();

    let pos = Position::new(
        RoomCoordinate::new(4).unwrap(),
        RoomCoordinate::new(4).unwrap(),
        room.name(),
    );
    let style = MapTextStyle::default()
        .align(screeps::TextAlign::Center)
        .font_size(7.0);

    if memory.rooms.contains_key(&room.name()) {
        MapVisual::text(pos, "🏠".to_string(), style);
    } else if memory.remote_rooms.contains_key(&room.name()) {
        MapVisual::text(pos, "🔋".to_string(), style);
    } else {
        MapVisual::text(pos, "👁️".to_string(), style);
    }

    if !room.my() && memory.rooms.contains_key(&room.name()) {
        memory.rooms.remove(&room.name());

        return;
    }

    if memory.last_generated_pixel + utils::ticks_to_fill_bucket(1000) <= game::time() {
        if !room.my()
            && !memory.remote_rooms.contains_key(&room.name())
            && game::cpu::bucket() < 1000
        {
            //info!("[{}] Skipping execution, bucket is too low...", room.name());

            return;
        } else if memory.remote_rooms.contains_key(&room.name()) && game::cpu::bucket() < 1000 {
            info!(
                "[REMOTE] Room {} running in low-power mode, to fix some bugs...",
                room.name()
            );

            cache.create_if_not_exists(&room, memory, None);
            organizer::run_creeps(&room, memory, cache);

            cache.non_owned_cpu +=
                (game::cpu::get_used() - starting_cpu) - (cache.creep_cpu - starting_creep_cpu);
            cache.non_owned_count += 1;

            return;
        }
    }

    if room.my() && !memory.rooms.contains_key(&room.name()) && !plan_room(&room, memory, cache) {
        return;
    }

    visualise_room_visual(&room.name());
    // Caches various things, like resources
    // Caches structures and other creep things
    cache.create_if_not_exists(&room, memory, None);

    // Ensure remote caches are created, just incase a remote harvester
    // Is sitting in this room, and the remote hasnt been run yet.
    // Should be cheap enough. (hopefully)
    if room.my() {
        if let Some(room_memory) = memory.rooms.get_mut(&room.name()) {
            let remotes = room_memory.remotes.clone();

            for remote in remotes {
                if let Some(game_room) = game::rooms().get(remote) {
                    cache.create_if_not_exists(&game_room, memory, Some(room.name()));
                }
            }
        }
    }

    if let Some(flag) = game::flags().get("distanceTransform".to_string()) {
        if flag.pos().room_name() == room.name() {
            let available_positions = distance_transform(&room.name(), None, true);
        }
    }

    #[cfg(feature = "season1")]
    //resources::haul_score_resources(&room.name(), cache, memory);

    if room.my() {
        info!("[GOVERNMENT] Starting government for room: {}", room.name());

        if memory.rooms.get(&room.name()).unwrap().rcl != room.controller().unwrap().level()
            || game::time() % 3000 == 0
        {
            if let Some(path_heap) = heap().flow_cache.lock().unwrap().get_mut(&room.name()) {
                path_heap.storage = None;

                path_heap.paths.clear();
            }
        }

        if game::cpu::bucket() >= 5000 {
            run_planner(&room, memory.rooms.get_mut(&room.name()).unwrap());
        }

        if let Some(flag) = game::flags().get("cleanRoads".to_string()) {
            if flag.pos().room_name() == room.name() {
                clean_rooms_roads(&room, memory, cache);
            }
        }

        if !memory.rooms.contains_key(&room.name()) || !cache.rooms.contains_key(&room.name()) {
            return;
        }

        {
            let cached_room = cache.rooms.get_mut(&room.name()).unwrap();


            if should_safemode(&room, cached_room, memory) {
                if let Some(controller) = &cached_room.structures.controller {
                    controller.safe_mode();
                }
            }

            // Check for dropped resources, making requests for each
            // Run haulers to process the requests generated by structures earlier
            resources::haul_containers(cached_room);
            resources::haul_dropped_resources(cached_room);
            hauling::haul_extensions(cached_room);
            hauling::haul_ruins(cached_room);
            hauling::haul_tombstones(cached_room);
            hauling::haul_storage(cached_room);
            hauling::haul_spawn(cached_room);

            links::balance_links(&room, cached_room);

            if let Some(heap) = heap()
                .flow_cache
                .lock()
                .unwrap()
                .get_mut(&cached_room.room.name())
            {
                // TODO: Make this take plans into consideration
                // So we can reduce the amount of recalculations
                if game::time() % 2000 == 0 {
                    heap.storage = None;
                }
            }

            // Run creeps and other structures
            // Does NOT run haulers, as they need to be done last
            // Reasoning for this decision is below
            tower::run_towers(cached_room);
        }

        // Makes hauling requests for the rooms remotes :)
        resources::haul_remotes(&room, memory, cache);

        organizer::run_creeps(&room, memory, cache);

        let pre_hauler_cpu = game::cpu::get_used();

        if let Some(room) = cache.rooms.get_mut(&room.name()) {
            room.stats.cpu_creeps += game::cpu::get_used() - pre_hauler_cpu;
        }

        if let Some(hauler_cpu_usage) = cache
            .rooms
            .get_mut(&room.name())
            .unwrap()
            .stats
            .cpu_usage_by_role
            .get_mut(&Role::Hauler)
        {
            *hauler_cpu_usage += game::cpu::get_used() - pre_hauler_cpu;
        }

        if let Some(flag) = game::flags().get("resetRoadPlans".to_string()) {
            if flag.room().unwrap().name() == room.name() {
                memory.rooms.get_mut(&room.name()).unwrap().planned_paths.clear();
            }
        }

        if let Some(flag) = game::flags().get("visualiseRoads".to_string()) {
            if flag.room().unwrap().name() == room.name() {
                let all_paths = roads::get_all_cached_positions(&room.name(), memory);

                for (room_name, spots) in all_paths {
                    if let Some(room) = game::rooms().get(room_name) {
                        let vis = room.visual();

                        let assembled: Vec<(f32, f32)> = spots.iter().map(|spot| (spot.x().u8() as f32, spot.y().u8() as f32)).collect();

                        for spot in assembled {
                            vis.circle(spot.0, spot.1, None);
                        }
                    }
                }
            }
        }

        // TODO: why are we looping here?
        for flag in game::flags().values() {
            if flag.name().starts_with("forceSpawnCenter") {
                let pos = flag.pos();

                if flag.pos().room_name() == room.name() {
                    let room_cache = cache.rooms.get_mut(&room.name()).unwrap();
                    let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

                    room_cache.spawn_center = Some(pos.xy());
                    room_memory.spawn_center = pos.xy();
                }
            } else if flag.name().starts_with("forceStorageCenter") {
                let pos = flag.pos();

                if flag.pos().room_name() == room.name() {
                    let room_cache = cache.rooms.get_mut(&room.name()).unwrap();
                    let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

                    room_cache.storage_center = Some(pos.xy());
                    room_memory.storage_center = pos.xy();
                }
            }
        }

        // Recover creeps not existing in memory
        // Only works for haulers as of right now
        // All other un-recoverables get suicided
        if game::time() % 10 == 0 {
            recover_creeps(memory);
        }

        {
            run_crap_planner_code(&room, memory, cache);

            let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

            run_full_visuals(&room, memory, room_cache);

            let mut lifetime = 0;
            {
                lifetime = *heap().heap_lifetime.lock().unwrap();
            }

            let room_memory = memory.rooms.get_mut(&room.name()).unwrap();
            room_cache.stats.spawning_stats(&mut room_cache.structures);

            // If we dont have enough remotes, scan every 10 ticks
            // If its 30000 ticks, scan
            // If we just pushed code, or the heap reset, scan.
            if ((room_memory.remotes.len() < config::REMOTES_FOR_RCL(room_cache).into()
                && game::time() % 10 == 0)
                || game::time() % REMOTE_SCAN_FOR_RCL(room_cache) == 0
                || lifetime == 0)
                && game::cpu::bucket() > 500
            {
                let remotes = remotes::fetch_possible_remotes(&room, memory, cache);
                info!(
                    "  [REMOTES] Remote re-scan triggered, found {} remotes",
                    remotes.len()
                );
            }
        }
    } else {
        // Room is NOT mine, therefore we should run creeps
        // Traffic is run on every room, so no need to put it here
        organizer::run_creeps(&room, memory, cache);

        // Place remote containers every 250 ticks, if we have remotes.
        if game::time() % 250 == 0 && memory.remote_rooms.contains_key(&room.name()) {
            plan_remote_containers(&room, memory, cache);
        }

        if let Some(remote_memory) = memory.remote_rooms.get_mut(&room.name()) {
            if remote_memory.last_attack_time.is_none() {
                remote_memory.under_attack = false;
                memory.goals.remote_defense.remove(&remote_memory.name);
            }

            if remote_memory.last_attack_time.is_some()
                && remote_memory.last_attack_time.unwrap() + 1000 < game::time()
            {
                if let Some(remote_cache) = cache.rooms.get(&remote_memory.name) {
                    if remote_cache.creeps.enemy_creeps.is_empty() {
                        remote_memory.under_attack = false;
                        remote_memory.last_attack_time = None;
                    }
                }
            }

            if !memory.rooms.contains_key(&remote_memory.owner) {
                memory.remote_rooms.remove(&room.name());
            }
        }

        if let Some(scouted_room) = memory.scouted_rooms.get(&room.name()) {
            if scouted_room.last_scouted < game::time() - 100 {
                rank_room::scout_room(&room, memory, cache.rooms.get_mut(&room.name()).unwrap());
            }
        } else {
            rank_room::scout_room(&room, memory, cache.rooms.get_mut(&room.name()).unwrap());
        }
    }

    hate_handler::process_room_event_log(&room, memory, cache);

    // Match these haulers to their tasks, that way we can run them
    // This is top-down now, got moved to the lib.rs
    //room_cache.hauling.match_haulers(memory, &room.name());

    if room.my() {
        let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

        let controller = room.controller().unwrap();
        room_cache.stats.rcl = controller.level();
        room_cache.stats.rcl_progress = controller.progress();
        room_cache.stats.rcl_progress_total = controller.progress_total();

        room_cache
            .stats
            .write_to_memory(memory, room.name(), game::cpu::get_used() - starting_cpu);

        info!(
            "  [GOVERNMENT] Finished government for room: {} - CPU: {:.2}",
            room.name(),
            game::cpu::get_used() - starting_cpu
        );
    } else {
        cache.non_owned_cpu +=
            (game::cpu::get_used() - starting_cpu) - (cache.creep_cpu - starting_creep_cpu);
        cache.non_owned_count += 1;
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn remote_path_call(_room_name: RoomName) -> MultiRoomCostResult {
    let matrix = LocalCostMatrix::new();
    MultiRoomCostResult::CostMatrix(matrix.into())
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_crap_planner_code(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    if game::cpu::bucket() < 500 {
        return;
    }

    if game::cpu::bucket() > 1000 && game::time() % 100 == 0 {
        let room_memory = memory.rooms.get_mut(&room.name()).unwrap();
        let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

        let should_rampart = room_cache.structures.storage.is_some() && room_cache.rcl >= 4;
        let mut should_road = room_cache.rcl >= 3;

        if !room_memory.planned
            || (room_memory.rcl != room.controller().unwrap().level())
            || game::time() % 300 == 0
        {
            heap()
                .cachable_positions
                .lock()
                .unwrap()
                .remove(&room.name());
            heap().flow_cache.lock().unwrap().remove(&room.name());

            let level = room.controller().unwrap().level();

            room_memory.rcl_times.insert(level, game::time());

            let structures = match room.controller().unwrap().level() {
                2 => get_rcl_2_plan(),
                3 => get_rcl_3_plan(),
                4 => get_rcl_4_plan(),
                5 => get_rcl_5_plan(),
                6 => get_rcl_6_plan(),
                7 => get_rcl_7_plan(),
                8 => get_rcl_8_plan(),
                _ => get_roads_and_ramparts(),
            };

            for structure in structures {
                if room_cache.structures.spawns.is_empty() {
                    continue;
                }

                if !should_rampart && structure.2 == StructureType::Rampart {
                    continue;
                }

                if !should_road && structure.2 == StructureType::Road {
                    continue;
                }

                let offset_x = room_cache.spawn_center.unwrap().x.u8();
                let offset_y = room_cache.spawn_center.unwrap().y.u8() + 1;

                let pos = RoomPosition::new(
                    structure.0 as u8 + offset_x,
                    structure.1 as u8 + offset_y,
                    room.name(),
                );
                let r = room.ITcreate_construction_site(pos.x(), pos.y(), structure.2, None);

                if r.is_ok() {
                    heap().flow_cache.lock().unwrap().remove(&room.name());
                }
            }

            // Plan container around source and controller
            let controller = &room_cache.structures.controller;
            let sources = &room_cache.resources.sources;

            let cp = controller.as_ref().unwrap().pos();
            let controller_looked = room.look_for_at_area(
                look::TERRAIN,
                cp.y().u8() - 2,
                cp.x().u8() - 2,
                cp.y().u8() + 2,
                cp.x().u8() + 2,
            );

            for pos in controller_looked {
                if let LookResult::Terrain(terrain) = pos.look_result {
                    if Terrain::Plain != terrain || Terrain::Swamp != terrain {
                        continue;
                    }

                    let pos = RoomPosition::new(pos.x, pos.y, room.name());
                    if pos.get_range_to_xy(cp.x().u8(), cp.y().u8()) != 2 {
                        continue;
                    }

                    let _ = room.ITcreate_construction_site(
                        pos.x(),
                        pos.y(),
                        StructureType::Container,
                        None,
                    );
                    break;
                }
            }

            for source in sources {
                let x = source.source.pos().x().u8();
                let y = source.source.pos().y().u8();

                let looked = room.look_for_at_area(look::TERRAIN, y - 1, x - 1, y + 1, x + 1);
                for pos in looked {
                    if let LookResult::Terrain(terrain) = pos.look_result {
                        if Terrain::Plain != terrain || Terrain::Swamp != terrain {
                            continue;
                        }
                        let res = room.ITcreate_construction_site(
                            pos.x,
                            pos.y,
                            StructureType::Container,
                            None,
                        );
                        if res.is_ok() {
                            break;
                        }
                    }
                }
            }
            room_memory.planned = true;
            room_memory.rcl = room.controller().unwrap().level();
        }

        let stuffs = get_roads_and_ramparts();

        let pos = room_cache.spawn_center.unwrap();

        let offset_x = pos.x;
        let offset_y = unsafe { RoomCoordinate::unchecked_new(pos.y.u8() + 1) };

        let room_memory = memory.rooms.get_mut(&room.name()).unwrap();

        if let Some(flag) = game::flags().get("deleteAllRoadCSites".to_string()) {
            let csites = game::construction_sites()
                .values()
                .filter(|cs| cs.structure_type() == StructureType::Road)
                .collect::<Vec<screeps::ConstructionSite>>();

            for csite in csites {
                let _ = csite.remove();
            }
        }

        if let Some(flag) = game::flags().get("deleteAllRoads".to_string()) {
            let room_cache = cache.rooms.get(&room.name()).unwrap();

            for road in room_cache.structures.roads.values() {
                road.destroy();
            }
        }

        let mut road_count = game::construction_sites()
            .values()
            .filter(|cs| cs.structure_type() == StructureType::Road)
            .count();

        info!(
            "[PLANNER]  Planning roads for room: {} - Count: {}",
            room.name(),
            road_count
        );

        if road_count > 50 {
            should_road = false;
        }

        if should_road {
            let mut should_plan = false;

            for remote in room_memory.remotes.iter() {
                if !room_memory.planned_paths.contains_key(remote) {
                    should_plan = true;
                    break;
                }
            }

            if room_memory.planned_paths.is_empty() || should_plan {
                let res = plan_main_room_roads(room, cache, memory);
                memory.rooms.get_mut(&room.name()).unwrap().planned_paths = res;
            }

            let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

            if room_cache.rcl >= 2 {
                planning::room::construction::plan_containers_and_links(room, room_cache);
            }

            let planned_paths = memory
                .rooms
                .get(&room.name())
                .unwrap()
                .planned_paths
                .clone();

            if let Some(owning_room) = planned_paths.get(&room.name()) {
                for pos in decode_pos_list(owning_room.to_string()) {
                    if road_count >= 50 {
                        break;
                    }

                    road_count += 1;
                    let _ = room.ITcreate_construction_site(
                        pos.x().u8(),
                        pos.y().u8(),
                        StructureType::Road,
                        None,
                    );
                }
            }

            /*
            for (room_name, path) in memory.rooms.get(&room.name()).unwrap().planned_paths.iter() {
                if let Some(game_room) = game::rooms().get(*room_name) {
                    for pos in decode_pos_list(path.to_string()) {
                        if road_count >= 50 {
                            break;
                        }

                        road_count += 1;
                        let _ = game_room.ITcreate_construction_site(pos.x().u8(), pos.y().u8(), StructureType::Road, None);
                    }
                }
            }*/
        }

        for structure in get_containers() {
            let pos = RoomPosition::new(
                structure.0 as u8 + offset_x.u8(),
                structure.1 as u8 + offset_y.u8(),
                room.name(),
            );
            let r = room.ITcreate_construction_site(pos.x(), pos.y(), structure.2, None);

            if r.is_ok() {
                heap().flow_cache.lock().unwrap().remove(&room.name());
            }
        }

        let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

        for structure in stuffs {
            if !should_rampart && structure.2 == StructureType::Rampart {
                continue;
            }

            if !should_road && road_count < 50 && structure.2 == StructureType::Road {
                continue;
            }

            if structure.2 == StructureType::Road {
                road_count += 1;
            }

            let pos = RoomPosition::new(
                structure.0 as u8 + offset_x.u8(),
                structure.1 as u8 + offset_y.u8(),
                room.name(),
            );
            let r = room.ITcreate_construction_site(pos.x(), pos.y(), structure.2, None);

            if r.is_ok() {
                heap().flow_cache.lock().unwrap().remove(&room.name());
            }
        }

        if room_cache.rcl >= 6 && room_cache.structures.extractor.is_none() {
            if let Some(mineral) = &room_cache.resources.mineral {
                let pos = mineral.pos();
                let _ = room.ITcreate_construction_site(
                    pos.x().u8(),
                    pos.y().u8(),
                    StructureType::Extractor,
                    None,
                );
            }
        }
    }
}

pub fn clean_rooms_roads(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let all_roads = get_all_cached_road_positions(&room.name(), memory);
    let t = get_roads_and_ramparts();
    let bunker_roads = t.iter().filter(|s| s.2 == StructureType::Road).collect::<Vec<_>>();

    let room_cache = cache.rooms.get(&room.name()).unwrap();

    let offset_x = room_cache.spawn_center.unwrap().x.u8();
    let offset_y = room_cache.spawn_center.unwrap().y.u8() + 1;

    let mut all_planned_roads = Vec::new();
    for road in bunker_roads {
        all_planned_roads.push(new_xy((road.0 + offset_x as i8) as u8, (road.1 + offset_y as i8) as u8));
    }

    if let Some(planned_roads) = all_roads.get(&room.name()) {
        for road in planned_roads {
            all_planned_roads.push(new_xy(road.x().u8(), road.y().u8()))
        }
    }

    for road in room_cache.structures.roads.values() {
        let pos = road.pos().xy();

        if !all_planned_roads.contains(&pos) {
            road.destroy();
        }
    }
}