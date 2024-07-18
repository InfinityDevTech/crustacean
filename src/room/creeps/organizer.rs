use std::collections::HashMap;

use log::info;
use screeps::{game, Room, SharedCreepProperties};

use crate::{
    combat::hate_handler::process_health_event,
    memory::{Role, ScreepsMemory},
    room::{
        cache::{
            heap_cache::{HealthChangeType, HeapCreep},
            tick_cache::RoomCache,
        },
        creeps::{global, remote},
    },
    traits::{
        creep::CreepExtensions, intents_tracking::CreepExtensionsTracking, room::RoomExtensions,
    },
};

use super::{combat, local};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn run_creeps(room: &Room, memory: &mut ScreepsMemory, cache: &mut RoomCache) -> f64 {
    let starting_cpu = game::cpu::get_used();
    let pre_creeps_cpu = game::cpu::get_used();
    let mut cpu_usage_by_role = HashMap::new();
    let mut creeps_by_role = HashMap::new();

    // This is done in this manner to stop an "impossible" state
    // I reached, idk how, idk why, idk who, but it happened
    // and this is the only way I could think of to fix it
    let creeps = &cache.rooms.get_mut(&room.name()).unwrap().creeps.creeps_in_room.clone();

    if creeps.len() == 0 {
        return game::cpu::get_used() - starting_cpu;
    }

    if room.my() {
        info!("  [CREEPS] Running {} creeps", creeps.len());
    }

    let creep_count = creeps.len();

    let mut highest_user: String = "".to_string();
    let mut highest_usage: f64 = 0.0;

    for creep_name in creeps.keys() {
        let start_time = game::cpu::get_used();

        let creep = game::creeps().get(creep_name.to_string()).unwrap();
        let mut role = Role::Recycler;

        if let Some(creep_memory) = memory.creeps.get(&creep.name()) {
            role = creep_memory.role;

            cache.create_if_not_exists(
                &game::rooms().get(creep_memory.owning_room).unwrap(),
                memory,
                None,
            );
        }

        if memory.remote_rooms.contains_key(&room.name()) && game::cpu::bucket() < 1000 {
            match role {
                Role::RemoteHarvester => remote::remote_harvester::run_remoteharvester(&creep, memory, cache),
                Role::RemoteDefender => remote::remote_defender::run_remotedefender(&creep, memory, cache),
                Role::Bulldozer => combat::bulldozer::run_bulldozer(&creep, memory, cache),
                _ => { continue; }
            }

            info!("  [CREEPS] Bucket too low to run creeps, running essential remote roles");

            continue;
        }

        // Fucks up harvester spawning. Should be done per-creep.
        //if creep.spawning() { continue; }

        match role {
            Role::Harvester => local::harvester::run_harvester(&creep, memory, cache),
            Role::Hauler => local::hauler::run_hauler(&creep, memory, cache),
            Role::Repairer => local::repairer::run_repairer(&creep, memory, cache),
            Role::BaseHauler => local::base_hauler::run_basehauler(&creep, memory, cache),
            Role::StorageSitter => local::storage_sitter::run_storagesitter(&creep, memory, cache),
            Role::Upgrader => local::upgrader::run_upgrader(&creep, memory, cache),
            Role::Builder => local::builder::run_builder(&creep, memory, cache),
            Role::FastFiller => local::fast_filler::run_fastfiller(&creep, memory, cache),
            Role::Bulldozer => combat::bulldozer::run_bulldozer(&creep, memory, cache),
            Role::Scout => global::scout::run_scout(&creep, memory, cache),
            Role::RemoteHarvester => remote::remote_harvester::run_remoteharvester(&creep, memory, cache),

            Role::ExpansionBuilder => global::expansion_builder::run_expansionbuilder(&creep, memory, cache),

            Role::Claimer => global::claimer::run_claimer(&creep, memory, cache),
            Role::Unclaimer => global::unclaimer::run_unclaimer(&creep, memory, cache),
            Role::Recycler => global::recycler::run_recycler(&creep, memory, cache),
            Role::PhysicalObserver => global::physical_observer::run_physical_observer(&creep, memory, cache),

            Role::Reserver => combat::reserver::run_reserver(&creep, memory, cache),

            Role::RemoteDefender => remote::remote_defender::run_remotedefender(&creep, memory, cache),

            _ => {
                creep.bsay("BAD ROLE", true);
                global::recycler::run_recycler(&creep, memory, cache);
            }
        }

        let end_time = game::cpu::get_used();
        let cpu_used = end_time - start_time;

        // TODO: Make this use an average, im sick of creeps randomly dieing
        //if cpu_used > 12.0 && role != Role::Scout && role != Role::Bulldozer && role != Role::Harvester && role != Role::RemoteHarvester {
        //    info!(
        //        "  [CREEPS] Suiciding {} due to high CPU usage: {}",
        //        creep.name(),
        //        cpu_used
        //    );
        //
        //    let _ = creep.ITsuicide();
        //}

        if cpu_used > highest_usage {
            highest_usage = cpu_used;
            highest_user = creep.name();
        }
        let end_time = game::cpu::get_used();

        if let Some(role) = cpu_usage_by_role.get_mut(&role) {
            *role += end_time - start_time;
        } else {
            cpu_usage_by_role.insert(role, end_time - start_time);
        }

        if let Some(role) = creeps_by_role.get_mut(&role) {
            *role += 1;
        } else {
            creeps_by_role.insert(role, 1);
        }
    }

    let cached_room = cache.rooms.get_mut(&room.name()).unwrap();
    let creeps = &cached_room.creeps.creeps_in_room;

    for creep in creeps.values() {
            let heap_creep = cached_room
            .heap_cache
            .creeps
            .entry(creep.name())
            .or_insert_with(|| HeapCreep::new(creep));

        let health_change = heap_creep.get_health_change(creep);
        if health_change != HealthChangeType::None {
            process_health_event(creep, memory, health_change);
        }
    }

    let room_cache = cache.rooms.get_mut(&room.name()).unwrap();
    room_cache.stats.creep_count = creep_count as u32;
    room_cache.stats.cpu_usage_by_role = cpu_usage_by_role;
    room_cache.stats.creeps_by_role = creeps_by_role;

    let end_cpu = game::cpu::get_used();
    if room.my() {
        info!(
            "  [CREEPS] Used {:.4} CPU to run creeps {:.4} CPU per creep",
            end_cpu - starting_cpu,
            (end_cpu - starting_cpu) / creep_count as f64
        );
        info!(
            "  [CREEPS] Highest CPU usage: {:.4} by {}",
            highest_usage, highest_user
        );
    } else {
        info!(
            "  [{}] Used {:.4} CPU to run creeps {:.4} CPU per creep - Highest: {} : {:.4}",
            room.name(),
            end_cpu - starting_cpu,
            (end_cpu - starting_cpu) / creep_count as f64,
            highest_user,
            highest_usage
        );
    }

    if let Some(room) = cache.rooms.get_mut(&room.name()) {
        room.stats.cpu_creeps += game::cpu::get_used() - pre_creeps_cpu;
    }

    end_cpu - starting_cpu
}
