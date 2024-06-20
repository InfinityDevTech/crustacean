use log::info;
use screeps::{
    game,
    look::{self, LookResult},
    pathfinder::MultiRoomCostResult,
    HasPosition, LocalCostMatrix, Room, RoomName, RoomPosition, RoomXY, StructureType, Terrain,
};

use crate::{
    combat::{hate_handler, rank_room},
    memory::{Role, ScreepsMemory},
    room::{
        cache::tick_cache::{hauling, resources, traffic, RoomCache}, creeps::{local::hauler, organizer, recovery::recover_creeps}, planning::room::{plan_room, remotes, structure_visuals::RoomVisualExt}, spawning, tower, visuals::run_full_visuals
    },
    traits::room::RoomExtensions,
};

use super::{
    cache::{self, tick_cache::CachedRoom},
    planning::room::construction::{
            get_rcl_2_plan, get_rcl_3_plan, get_rcl_4_plan, get_rcl_5_plan, get_rcl_6_plan,
            get_rcl_7_plan, get_rcl_8_plan, get_roads_and_ramparts,
        }, visuals::visualise_room_visual,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn start_government(room: Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) {
    let starting_cpu = game::cpu::get_used();

    if !room.my() && game::cpu::bucket() < 1000 {
        info!("[{}] Skipping execution, bucket is too low...", room.name());
    }

    visualise_room_visual(&room.name());
    // Caches various things, like resources
    // Caches structures and other creep things
    cache.create_if_not_exists(&room, memory, None);

    if room.my() {
        cache.my_rooms.push(room.name());
        info!("[GOVERNMENT] Starting government for room: {}", room.name());

        if !memory.rooms.contains_key(&room.name()) && !plan_room(&room, memory, cache) {
            return;
        }

        {
            let cached_room = cache.rooms.get_mut(&room.name()).unwrap();
            // Check for dropped resources, making requests for each
            // Run haulers to process the requests generated by structures earlier
            resources::haul_containers(cached_room);
            resources::haul_dropped_resources(cached_room);
            hauling::haul_extensions(cached_room);
            hauling::haul_ruins(cached_room);
            hauling::haul_tombstones(cached_room);
            hauling::haul_storage(cached_room);
            hauling::haul_spawn(cached_room);

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

        if let Some(hauler_cpu_usage) = cache.rooms.get_mut(&room.name()).unwrap().stats.cpu_usage_by_role.get_mut(&Role::Hauler) {
            *hauler_cpu_usage += game::cpu::get_used() - pre_hauler_cpu;
        }

        // Recover creeps not existing in memory
        // Only works for haulers as of right now
        // All other un-recoverables get suicided
        if game::time() % 10 == 0 {
            recover_creeps(memory);
        }

        {
            let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

            run_crap_planner_code(&room, memory, room_cache);
            run_full_visuals(&room, memory, room_cache);

            if memory.rooms.get(&room.name()).unwrap().remotes.len() < 2 || game::time() % 3000 == 0 {
                let remotes = remotes::fetch_possible_remotes(&room, memory, room_cache);

                info!("Setting remotes for room: {} - {:?}", room.name(), remotes);
            }

            room_cache.stats.spawning_stats(&mut room_cache.structures);
        }
    } else {
        // Room is NOT mine, therefore we should run creeps
        // Traffic is run on every room, so no need to put it here
        organizer::run_creeps(&room, memory, cache);

        if let Some(scouted_room) = memory.scouted_rooms.get(&room.name()) {
            if scouted_room.last_scouted < game::time() - 100 {
                rank_room::rank_room(&room, memory, cache.rooms.get_mut(&room.name()).unwrap());
            }
        } else {
            rank_room::rank_room(&room, memory, cache.rooms.get_mut(&room.name()).unwrap());
        }
    }

    hate_handler::process_room_event_log(&room, memory, cache);

    let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

    // Match these haulers to their tasks, that way we can run them
    //room_cache.hauling.match_haulers(memory, &room.name());

    let start = game::cpu::get_used();
    traffic::run_movement(room_cache);

    if room.my() {
        info!(
            "  [TRAFFIX] Traffic took: {:.4} with {} intents",
            game::cpu::get_used() - start,
            room_cache.traffic.move_intents
        );
    }
    room_cache.write_cache_to_heap(&room);

    if room.my() {
        let end_cpu = game::cpu::get_used();
        let room_cache = cache.rooms.get_mut(&room.name()).unwrap();

        let controller = room.controller().unwrap();
        room_cache.stats.rcl = controller.level();
        room_cache.stats.rcl_progress = controller.progress();
        room_cache.stats.rcl_progress_total = controller.progress_total();

        room_cache.stats.write_to_memory(memory, room.name(), end_cpu - starting_cpu);
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn remote_path_call(room_name: RoomName) -> MultiRoomCostResult {
    let mut matrix = LocalCostMatrix::new();
    let terrain = game::map::get_room_terrain(room_name);

    if let Some(terrain) = terrain {
        for x in 0..50 {
            for y in 0..50 {
                let tile = terrain.get(x, y);
                let xy = unsafe { RoomXY::unchecked_new(x, y) };
                match tile {
                    Terrain::Plain => {
                        matrix.set(xy, 1);
                    }
                    Terrain::Swamp => {
                        matrix.set(xy, 5);
                    }
                    Terrain::Wall => {
                        matrix.set(xy, 255);
                    }
                }
            }
        }
    }
    MultiRoomCostResult::CostMatrix(matrix.into())
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_crap_planner_code(room: &Room, memory: &mut ScreepsMemory, room_cache: &CachedRoom) {
    let _coords = room_cache.structures.spawns.values().next().unwrap().pos();
    let _viz = RoomVisualExt::new(room.name());

    if game::cpu::bucket() < 500 {
        return;
    }

    if !memory.rooms.get(&room.name()).unwrap().planned
        || (memory.rooms.get(&room.name()).unwrap().rcl != room.controller().unwrap().level())
    {
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
            let offset_x = room_cache
                .structures
                .spawns
                .values()
                .next()
                .unwrap()
                .pos()
                .x();
            let offset_y = room_cache
                .structures
                .spawns
                .values()
                .next()
                .unwrap()
                .pos()
                .y();

            let pos = RoomPosition::new(
                structure.0 as u8 + offset_x.u8(),
                structure.1 as u8 + offset_y.u8(),
                room.name(),
            );
            let _ = room.create_construction_site(pos.x(), pos.y(), structure.2, None);
        }

        // Plan container around source and controller
        let controller = &room_cache.structures.controller;
        let sources = &room_cache.resources.sources;

        let cp = controller.as_ref().unwrap().controller.pos();
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

                let _ =
                    room.create_construction_site(pos.x(), pos.y(), StructureType::Container, None);
                break;
            }
        }

        for source in sources {
            let source = game::get_object_by_id_typed(&source.id).unwrap();

            let x = source.pos().x().u8();
            let y = source.pos().y().u8();

            let looked = room.look_for_at_area(look::TERRAIN, y - 1, x - 1, y + 1, x + 1);
            for pos in looked {
                if let LookResult::Terrain(terrain) = pos.look_result {
                    if Terrain::Plain != terrain || Terrain::Swamp != terrain {
                        continue;
                    }
                    let res =
                        room.create_construction_site(pos.x, pos.y, StructureType::Container, None);
                    if res.is_ok() {
                        break;
                    }
                }
            }
        }
        memory.rooms.get_mut(&room.name()).unwrap().planned = true;
        memory.rooms.get_mut(&room.name()).unwrap().rcl = room.controller().unwrap().level();
    }
}
