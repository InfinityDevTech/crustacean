use std::collections::HashMap;

use screeps::{Part, RoomName};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomClaimGoal {
    pub claim_target: RoomName,
    pub creeps_assigned: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomReservationGoal {
    pub reservation_target: RoomName,
    pub accessible_reservation_spots: u8,
    pub creeps_assigned: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteDefenseGoal {
    pub defending_remote: RoomName,
    pub power_rescan_tick: u32,
    pub total_attack_power: u32,
    pub attacker_names: Vec<String>,
    pub attacking_creeps: Vec<AttackingCreep>,
    pub creeps_assigned: Vec<String>,

    // Treats this so much differently
    // These fuckers are just an annoyance, so they arent high priority.
    pub invaders: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteInvaderCleanup {
    pub cleanup_target: RoomName,
    pub creeps_assigned: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttackingCreep {
    pub creep_name: String,
    pub owner_name: String,
    pub attack_power: u32,
    pub body: Vec<Part>,
    pub ttl: u32,
}

structstruck::strike! {
    #[strikethrough[derive(Serialize, Deserialize, Debug, Clone, Default)]]
    pub struct GoalMemory {
        pub room_claim: HashMap<RoomName, RoomClaimGoal>,
        pub room_reservation: HashMap<RoomName, RoomReservationGoal>,

        pub remote_defense: HashMap<RoomName, RemoteDefenseGoal>,
        pub remote_invader_cleanup: HashMap<RoomName, RemoteInvaderCleanup>,
    }
}