use std::collections::HashMap;

use screeps::{game, Part, RoomName};

use crate::memory::{self, Role, RoomStats, ScreepsMemory};

use super::structures::RoomStructureCache;

#[derive(Debug, Clone, Default)]
pub struct StatsCache {
    pub global_pathfinding: f64,


    pub rcl: u8,
    pub rcl_progress: Option<u32>,
    pub rcl_progress_total: Option<u32>,

    pub creep_count: u32,
    pub cpu_usage_by_role: HashMap<Role, f64>,
    pub creeps_by_role: HashMap<Role, u32>,

    pub cpu_creeps: f64,
    pub cpu_traffic: f64,
    pub cpu_cache: f64,
    pub cpu_hauling_orders: f64,

    pub energy: EnergyStats,
}

#[derive(Debug, Clone, Default)]
pub struct EnergyStats {
    pub capacity: u32,
    pub available: u32,
    pub stored: u32,

    pub income_mining: u32,
    pub income_trading: u32,
    pub income_other: u32,

    pub spending_spawning: u32,
    pub spending_upgrading: u32,
    pub spending_construction: u32,
    pub spending_repair: u32,
}

impl StatsCache {
    pub fn spawning_stats(&mut self, structures: &mut RoomStructureCache) {
        for spawn in structures.spawns.values() {
            if spawn.spawning().is_none() { continue; }

            let creep_name = spawn.spawning().unwrap().name();
            let time = spawn.spawning().unwrap().need_time();

            let parts = game::creeps().get(creep_name.as_string().unwrap()).unwrap().body().iter().map(|part| part.part()).collect::<Vec<Part>>();
            let body_cost = parts.iter().map(|part| part.cost()).sum::<u32>();

            self.energy.spending_spawning = body_cost / time;
        }
    }
    pub fn write_to_memory(&self, memory: &mut ScreepsMemory, room_name: RoomName, cpu_used: f64) {
        let room_stats = memory.stats.rooms.get_mut(&room_name);

        if let Some(room_stats) = room_stats {
            memory.stats.cpu.pathfinding += self.global_pathfinding;

            room_stats.rcl = self.rcl;
            room_stats.rcl_progress = self.rcl_progress;
            room_stats.rcl_progress_total = self.rcl_progress_total;

            room_stats.creep_count = self.creep_count;

            room_stats.energy.capacity = self.energy.capacity;
            room_stats.energy.available = self.energy.available;
            room_stats.energy.stored = self.energy.stored;

            room_stats.energy.income_mining = self.energy.income_mining;
            room_stats.energy.income_trading = self.energy.income_trading;
            room_stats.energy.income_other = self.energy.income_other;

            room_stats.energy.spending_spawning = self.energy.spending_spawning;
            room_stats.energy.spending_upgrading = self.energy.spending_upgrading;
            room_stats.energy.spending_construction = self.energy.spending_construction;
            room_stats.energy.spending_repair = self.energy.spending_repair;

            room_stats.cpu_used = cpu_used;
            room_stats.cpu_traffic = self.cpu_traffic;
            room_stats.cpu_creeps = self.cpu_creeps;
            room_stats.cpu_hauling_orders = self.cpu_hauling_orders;
            room_stats.cpu_cache = self.cpu_cache;

            room_stats.cpu_usage_by_role.clone_from(&self.cpu_usage_by_role);
            room_stats.creeps_by_role.clone_from(&self.creeps_by_role);
        } else {
            let energy = memory::EnergyStats {
                capacity: self.energy.capacity,
                available: self.energy.available,
                stored: self.energy.stored,

                income_mining: self.energy.income_mining,
                income_trading: self.energy.income_trading,
                income_other: self.energy.income_other,

                spending_spawning: self.energy.spending_spawning,
                spending_upgrading: self.energy.spending_upgrading,
                spending_construction: self.energy.spending_construction,
                spending_repair: self.energy.spending_repair,
            };

            let stats = RoomStats {
                rcl: self.rcl,
                rcl_progress: self.rcl_progress,
                rcl_progress_total: self.rcl_progress_total,

                cpu_creeps: self.cpu_creeps,
                cpu_traffic: self.cpu_traffic,
                cpu_cache: self.cpu_cache,
                cpu_hauling_orders: self.cpu_hauling_orders,

                cpu_used,
                energy,
                creeps_by_role: self.creeps_by_role.clone(),
                cpu_usage_by_role: self.cpu_usage_by_role.clone(),

                creep_count: self.creep_count,
            };

            memory.stats.rooms.insert(room_name, stats);
        }
    }
}