use std::{collections::HashMap, str::FromStr};

use js_sys::JsString;
use log::info;
use screeps::game;
use serde::{Deserialize, Serialize};

use crate::memory::ScreepsMemory;

const SYNC_SEGMENT: u8 = 99;
const SYNC_INTERVAL: u8 = 100;

pub fn get_sync_user(shard: &str) -> String {
    match shard {
        "shard0" => "".to_string(),
        "shard1" => "U-238".to_string(),
        "shard2" => "Winnduu".to_string(),
        "shard3" => "Shylo132".to_string(),
        _ => "".to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SSSSegmentAllySyncData {
    pub allies: HashMap<String, bool>,
}

pub struct SSSAllySync {
    pub allies: Vec<String>,
    pub enemies: Vec<String>,

    pub next_sync_time: u32,
}

impl SSSAllySync {
    pub fn new(memory: &mut ScreepsMemory) -> SSSAllySync {
        let allies = [
            "Shylo132",
            "MerlinMan5",
            "Starb",
            "PlainCucumber25",
            "DollarAkshay",
            "Pankpanther",
            "U-238",
            "Winnduu",
            "Salieri",
            "ChuChuChu",
            "Diesel13",
            "Loop_Cat"
        ];

        let enemies = [
            "ThomasCui"
        ];

        for ally in &allies {
            if !memory.allies.contains(&ally.to_string()) {
                memory.allies.push(ally.to_string());
            }
        }

        SSSAllySync {
            allies: allies.iter().map(|x| x.to_string()).collect(),
            enemies: enemies.iter().map(|x| x.to_string()).collect(),
            next_sync_time: game::time()
        }
    }

    pub fn sync(&mut self, memory: &mut ScreepsMemory) {
        if game::time() < self.next_sync_time {
            return;
        }

        let sync_player = get_sync_user(&game::shard::name());
        if sync_player.is_empty() { return };

        let segment = screeps::raw_memory::foreign_segment();
        if segment.is_none() || segment.as_ref().unwrap().username() != sync_player || segment.as_ref().unwrap().id() != SYNC_SEGMENT {
            // Set public segment for read and wait for next tick
            screeps::raw_memory::set_active_foreign_segment(&JsString::from_str(&sync_player).unwrap(), Some(SYNC_SEGMENT));
            if game::time() > self.next_sync_time + 2 {
                self.next_sync_time = game::time() + SYNC_INTERVAL as u32;
            }
            return;
        }

        let data = segment.unwrap().data();
        match serde_json::from_str::<SSSSegmentAllySyncData>(&data.as_string().unwrap()) {
            Ok(allies) => {
                info!("[SSS] Allies are synced!");
                for ally in allies.allies {
                    let is_ally = ally.1;

                    if is_ally {
                        if !self.allies.contains(&ally.0) {
                            self.allies.push(ally.0.clone());
                        }

                        if !memory.allies.contains(&ally.0) {
                            memory.allies.push(ally.0);
                        }
                    } else if !self.enemies.contains(&ally.0) {
                        self.enemies.push(ally.0);
                    }
                }

                self.next_sync_time = game::time() + SYNC_INTERVAL as u32;
            },
            Err(_) => {
                info!("[SSS] Failed to sync allies");
                // Womp womp
            },
        }
    }

    pub fn is_ally(&self, username: &str) -> bool {
        self.allies.contains(&username.to_string())
    }

    pub fn is_enemy(&self, username: &str) -> bool {
        self.enemies.contains(&username.to_string())
    }
}