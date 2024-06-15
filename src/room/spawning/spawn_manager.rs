use log::info;
use screeps::{creep, game, Part, Position, Room, RoomName, SpawnOptions, StructureSpawn};

use crate::{memory::{CreepMemory, Role, ScreepsMemory}, room::cache::tick_cache::CachedRoom, utils::role_to_name};

pub struct SpawnRequest {
    role: Role,
    body: Vec<Part>,
    priority: f64,
    cost: u32,

    creep_memory: CreepMemory,

    spawn_options: Option<SpawnOptions>
}

pub struct SpawnManager {
    room_name: RoomName,

    spawn_queue: Vec<SpawnRequest>,
    spawns: Vec<StructureSpawn>,
}

impl SpawnManager {
    pub fn new(room_name: &RoomName, cache: &mut CachedRoom) -> Self {
        Self {
            room_name: *room_name,

            spawn_queue: Vec::new(),
            spawns: cache.structures.spawns.values().cloned().collect(),
        }
    }

    pub fn create_spawn_request(&mut self, role: Role, body: Vec<Part>, priority: f64, cost: u32, creep_memory: Option<CreepMemory>, spawn_options: Option<SpawnOptions>) {
        let mut creep_memory = if let Some(creep_memory) = creep_memory {
            creep_memory
        } else {
            CreepMemory {
                owning_room: self.room_name,
                ..Default::default()
            }
        };

        if creep_memory.role != role {
            creep_memory.role = role;
        }


        let request = SpawnRequest {
            role,
            body,
            priority,
            cost,

            creep_memory,

            spawn_options
        };

        self.spawn_queue.push(request);
    }

    pub fn run_spawning(&mut self, room: &Room, memory: &mut ScreepsMemory) {
        if self.spawn_queue.is_empty() { return; }

        let available_spawns = self.get_available_spawns();
        if available_spawns.is_empty() { return; }

        // Sort the queue from highest to lowest
        self.spawn_queue.sort_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap());

        for spawn in available_spawns {
            let request = self.spawn_queue.pop().unwrap();

            info!("Top scoring role {:?}", request.role);

            if self.can_spawn(room, &request) {
                let options = request.spawn_options.unwrap_or_default();

                let role_name = role_to_name(request.role);
                let creep_name = format!("{}-{}-{}", role_name, room.name(), memory.get_id());

                memory.create_creep(&self.room_name, &creep_name, request.creep_memory);

                let _ = spawn.spawn_creep_with_options(&request.body, &creep_name, &options);
            }
        }
    }

    pub fn can_spawn(&mut self, room: &Room, request: &SpawnRequest) -> bool {
        let cost = request.cost;

        if room.energy_available() < cost {
            return false;
        }

        let available_spawn = self.get_available_spawns();
        if available_spawn.is_empty() {
            return false;
        }

        let options = SpawnOptions::default().dry_run(true);
        let dry_run_result = available_spawn.first().unwrap().spawn_creep_with_options(&request.body, "dry_run", &options);

        dry_run_result.is_ok()
    }

    pub fn get_available_spawns(&mut self) -> Vec<StructureSpawn> {
        let mut available_spawns = Vec::new();
        for spawn in &self.spawns {
            if spawn.spawning().is_none() {
                available_spawns.push(spawn.clone())
            }
        }

        available_spawns
    }
}