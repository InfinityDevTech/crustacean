use std::collections::HashMap;

use enum_map::{enum_map, Enum, EnumMap};
use log::error;
use screeps::{game, ObjectId, RawObjectId, ResourceType, RoomName, RoomXY, Source, Structure, StructureContainer, StructureLink};
use serde::{Deserialize, Serialize};

use js_sys::JsString;
//use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{config::MEMORY_VERSION, goal_memory::GoalMemory, room::{cache::hauling::HaulingType, expansion::ExpansionMemory}, traits::room::RoomType};

#[derive(Debug, Clone, Serialize, Deserialize, Enum)]
pub enum SegmentIDs {
    Profiler = 9
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn segment_ids() -> EnumMap<SegmentIDs, u8> {
    enum_map! {
        SegmentIDs::Profiler => 9,
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Copy, Enum)]
// The roles listed in creep memory
// The order of this also is the order in which
// Traffic Priority is handled.
pub enum Role {
    // Mining industry
    Harvester,
    MineralMiner,
    Hauler,

    FastFiller,
    BaseHauler,
    StorageSitter,

    Upgrader,
    Repairer,
    Scout,
    Builder,

    RemoteHarvester,
    PhysicalObserver,

    Bulldozer,
    Unclaimer,

    ExpansionBuilder,

    Claimer,
    Reserver,
    RemoteDefender,
    InvaderCoreCleaner,

    InvaderDuoAttacker,
    InvaderDuoHealer,

    #[cfg(feature = "season1")]
    Season1Digger,
    #[cfg(feature = "season1")]
    Season1Scorer,

    // Assorted junk roles, recycler just recycles itself
    Recycler,
    GiftBasket,
}

pub fn iter_roles() -> Vec<Role> {
    vec![
        Role::Harvester,
        Role::MineralMiner,
        Role::Hauler,

        Role::FastFiller,
        Role::BaseHauler,
        Role::StorageSitter,

        Role::Upgrader,
        Role::Repairer,
        Role::Scout,
        Role::Builder,

        Role::RemoteHarvester,
        Role::PhysicalObserver,

        Role::Bulldozer,
        Role::Unclaimer,

        Role::ExpansionBuilder,

        Role::Claimer,
        Role::Reserver,
        Role::RemoteDefender,
        Role::InvaderCoreCleaner,

        Role::InvaderDuoAttacker,
        Role::InvaderDuoHealer,

        Role::Recycler,
        Role::GiftBasket,
    ]
}
// What each creep stores in its memory.
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct CreepMemory{
    // Role, to allow role switching
    #[serde(rename = "20")]
    pub role: Role,

    // Owning room
    #[serde(rename = "0")]
    pub owning_room: RoomName,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "1")]
    pub owning_remote: Option<RoomName>,
    // Path
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "2")]
    pub path: Option<String>,
    // Needs Energy?
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "3")]
    pub needs_energy: Option<bool>,
    // This is miner specific, the ID of the link next to it
    // If this is empty, then the miner is not linked to a link and it will drop resources on the ground
    // If it is, but the link isnt next to it, the miner will clear the link id. If it is, the miner will deposit resources into the link
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "4")]
    pub link_id: Option<ObjectId<StructureLink>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "5")]
    pub fastfiller_container: Option<ObjectId<StructureContainer>>,
    // This is a pointer that changes based on the role of the creep
    // Miner - A reference to the source in the vec of sources
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "6")]
    pub task_id: Option<u128>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "8")]
    pub scout_target: Option<RoomName>,
    // The hauling task if a creep is a hauler.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "9")]
    pub hauling_task: Option<pub struct CreepHaulTask {
        #[serde(rename = "0")]
        pub target_id: RawObjectId,
        #[serde(rename = "1")]
        pub haul_type: HaulingType,
        #[serde(rename = "2")]
        pub priority: f32,
        #[serde(rename = "3")]
        pub resource: ResourceType,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "4")]
        pub amount: Option<u32>,
    }>,

    // Role specific memory ----------

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "19")]
    pub repair_target: Option<ObjectId<Structure>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "20")]
    pub is_recycling: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "21")]
    pub target_room: Option<RoomName>,
}
}

// Room Memory
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
pub struct RoomMemory{
    // Name
    pub name: RoomName,
    pub rcl: u8,
    pub max_rcl: u8,
    pub id: u128,
    pub planned: bool,
    pub chant_index: u64,
    // Creeps by role
    pub creeps: Vec<String>,
    pub remotes: Vec<RoomName>,

    pub rcl_times: HashMap<u8, u32>,

    pub spawn_center: RoomXY,
    pub storage_center: RoomXY,
    pub planned_paths: HashMap<RoomName, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skippy_planner: Option<pub struct SkippyMem {
        pub step: u8,
        pub map: Vec<char>,
        pub source_fills: HashMap<ObjectId<Source>, HashMap<i32, i32>>,
        pub controller_fill: HashMap<i32, i32>,
        pub orth_wall_fill: HashMap<i32, i32>,

        pub stamp_index: Option<i32>,
        pub source_labs: [i32; 2],
        pub core: i32,
        pub planned: bool,
    }>,

    pub avg_spawn_expense: f64,
    pub income: u32,
    pub expense: u32,
    pub under_attack: bool,
    pub hauler_count: u32,
}
}

// Remote Room memory
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct RemoteRoomMemory {
        pub name: RoomName,
        pub owner: RoomName,

        pub creeps: Vec<String>,
        pub sources: Vec<ScoutedSource>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_attack_time: Option<u32>,
        pub under_attack: bool,
    }
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct EnemyPlayer {
        pub username: String,
        pub hate: f32,

        pub owned_rooms: Vec<RoomName>,
        pub reserved_rooms: Vec<RoomName>,

        pub last_attack: u32,
    }
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone, Default)]]
    pub struct FormationMemory {
        pub duos: HashMap<u128, pub struct DuoMemory {
            pub creeps: Vec<String>,
        }>,
    }
}

// Scouted Room Data
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScoutedRoom {
        pub name: RoomName,
        pub room_type: RoomType,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rcl: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub owner: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub invader_core: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reserved: Option<String>,

        pub defense_capability: u8,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub sources: Option<Vec<pub struct ScoutedSource {
            pub pos: RoomXY,
            pub pos_av: u8,
        }>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub controller: Option<RoomXY>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub source_keepers: Option<Vec<RoomXY>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mineral: Option<ResourceType>,
        pub last_scouted: u32,
    }
}

// Stats
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone, Default)]]
    pub struct StatsData {
        pub gpl: u32,
        pub tick: u32,
        pub last_reset: u32,
        pub age: u32,

        pub gcl: pub struct GCLStats {
            pub level: u32,
            pub progress: f64,
            pub progress_total: f64,
        },

        pub market: pub struct MarketStats {
            pub credits: f64,
            pub cpu_unlocks: u32,
            pub access_keys: u32,
            pub pixels: u32,
        },

        pub memory_usage: pub struct MemoryStats {
            pub used: u32,
            pub total: u32,
        },

        pub heap_usage: pub struct Heapusage {
            pub total: u32,
            pub used: u32,
        },

        pub cpu: pub struct CPUStats {
            pub bucket: i32,
            pub used: f64,
            pub limit: u32,

            #[serde(default)]
            pub goal_creation: f64,
            #[serde(default)]
            pub goal_execution: f64,
            pub rooms: f64,
            pub memory: f64,
            pub market: f64,
            #[serde(default)]
            pub traffic_solving: f64,
            #[serde(default)]
            pub traffic_execution: f64,
            #[serde(default)]
            pub expansion: f64,
            #[serde(default)]
            pub hauler_matching: f64,
            #[serde(default)]
            pub spawning: f64,
            pub creeps: f64,
            pub pathfinding: f64,
        },

        pub rooms: HashMap<RoomName, pub struct RoomStats {
            pub cpu_used: f64,

            pub rcl: u8,
            pub rcl_progress: Option<u32>,
            pub rcl_progress_total: Option<u32>,

            pub creep_count: u32,
            pub cpu_usage_by_role: HashMap<Role, f64>,
            pub creeps_by_role: HashMap<Role, u32>,

            pub cpu_traffic: f64,
            pub cpu_creeps: f64,
            pub cpu_cache: f64,
            pub cpu_hauling_orders: f64,

            pub economy: pub struct EconomyStats {
                pub energy_capacity: u32,
                pub available_energy: u32,
                pub dropped_energy: u32,
                pub stored_energy: u32,
                pub stored_energy_in_haulers: u32,
                pub stored_energy_in_base_haulers: u32,
                pub stored_energy_in_containers: u32,

                pub income_energy: u32,
                pub income_minerals: u32,
                pub income_power: u32,
                pub income_trading: u32,
                pub income_other: u32,

                pub deposited_energy: u32,
                pub deposited_minerals: u32,

                pub spending_spawning: u32,
                pub spending_upgrading: u32,
                pub spending_construction: u32,
                pub spending_repair: u32,
            },
        }>
    }
}

// Top level memory.
structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone)]]
    pub struct ScreepsMemory {
        pub creep_say: bool,
        pub subtract_intents_profiler: bool,

        pub last_generated_pixel: u32,

        pub id_index: u128,
        pub mem_version: u8,
        pub rooms: HashMap<RoomName, RoomMemory>,
        pub remote_rooms: HashMap<RoomName, RemoteRoomMemory>,
        pub creeps: HashMap<String, CreepMemory>,
        pub formations: FormationMemory,

        pub goals: GoalMemory,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub expansion: Option<ExpansionMemory>,

        pub enemy_players: HashMap<String, EnemyPlayer>,
        pub scouted_rooms: HashMap<RoomName, ScoutedRoom>,

        pub stats: StatsData,
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl ScreepsMemory {
    pub fn init_memory() -> Self {
        let pre_memory_cpu = game::cpu::get_used();

        let memory_jsstring = screeps::raw_memory::get();
        let memory_string = memory_jsstring.as_string().unwrap();
        if memory_string.is_empty() {

            let mut memory = ScreepsMemory {
                creep_say: true,
                subtract_intents_profiler: true,

                last_generated_pixel: 0,

                id_index: 0,
                mem_version: MEMORY_VERSION,
                rooms: HashMap::new(),
                remote_rooms: HashMap::new(),
                creeps: HashMap::new(),
                formations: FormationMemory::default(),
                goals: GoalMemory::default(),
                expansion: None,

                enemy_players: HashMap::new(),
                scouted_rooms: HashMap::new(),
                stats: StatsData::default(),
            };

            memory.write_memory();

            memory.stats.cpu.memory = game::cpu::get_used() - pre_memory_cpu;
            memory
        } else {
            match serde_json::from_str::<ScreepsMemory>(&memory_string) {
                Ok(mut memory) => {
                    memory.stats.cpu.memory = game::cpu::get_used() - pre_memory_cpu;

                    memory
                },
                Err(e) => {
                    error!("Error parsing memory: {}", e);
                    error!("This is a critical error, memory MUST be reset to default state.");
                    error!("Memory: {}", memory_string);

                    let mut memory = ScreepsMemory {
                        creep_say: true,
                        subtract_intents_profiler: true,

                        last_generated_pixel: 0,

                        id_index: 0,
                        mem_version: MEMORY_VERSION,
                        rooms: HashMap::new(),
                        remote_rooms: HashMap::new(),
                        creeps: HashMap::new(),
                        formations: FormationMemory::default(),
                        goals: GoalMemory::default(),
                        expansion: None,

                        enemy_players: HashMap::new(),
                        scouted_rooms: HashMap::new(),
                        stats: StatsData::default(),
                    };

                    memory.stats.cpu.memory = game::cpu::get_used() - pre_memory_cpu;
                    memory.write_memory();
                    memory
                }
            }
        }
    }

    pub fn write_memory(&mut self) {
        let serialized = serde_json::to_string(&self).unwrap();
        let js_serialized = JsString::from(serialized);

        screeps::raw_memory::set(&js_serialized);
    }

    pub fn activate_segments(&self) {
        screeps::raw_memory::set_active_segments(&[segment_ids()[SegmentIDs::Profiler]]);
    }

    pub fn filter_old_creeps(&mut self) {
        for (creep_name, creep_memory) in &self.creeps.clone() {
            let owning_room = creep_memory.owning_room;
            if game::rooms().get(owning_room).is_none() {
                self.creeps.remove(creep_name);
            }
        }
    }

    pub fn create_creep(&mut self, room_name: &RoomName, creep_name: &str, object: CreepMemory) {
        self.creeps.insert(creep_name.to_string(), object);

        if let Some(room) = self.rooms.get_mut(room_name) {
            room.creeps.push(creep_name.to_string());
        }
    }

    pub fn create_room(&mut self, name: &RoomName, object: RoomMemory) {
        self.rooms.insert(
            *name,
            object
        );
    }

    pub fn create_scouted_room(&mut self, name: RoomName, object: ScoutedRoom) {
        self.scouted_rooms.insert(
            name,
            object
        );
    }
}

impl Default for CreepMemory {
    fn default() -> Self {
        CreepMemory {
            owning_room: RoomName::new("W0N0").unwrap(),
            role: Role::Recycler,
            owning_remote: None,
            path: None,
            needs_energy: None,
            link_id: None,
            fastfiller_container: None,
            task_id: None,

            target_room: None,
            repair_target: None,
            scout_target: None,
            hauling_task: None,
            is_recycling: None,
        }
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl EnemyPlayer {
    pub fn decrement_hate(&mut self, amount: f32) {
        let current_hate = self.hate;

        if current_hate - amount < 0.0 {
            self.hate = 0.0;
        } else {
            self.hate -= amount;
        }
    }

    pub fn increment_hate(&mut self, amount: f32) {
        self.hate += amount;
    }
}