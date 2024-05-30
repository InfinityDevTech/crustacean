use screeps::{game, Event, Room};

use crate::memory::ScreepsMemory;

pub struct EventLogProcessor {
    pub events: Vec<Event>,
}

impl EventLogProcessor {
    pub fn new_from_room(room: &Room) -> EventLogProcessor {
        let log = room.get_event_log();

        EventLogProcessor {
            events: log,
        }
    }

    pub fn add_enemy_player(&self)

    pub fn process_combat_events(&self, memory: &mut ScreepsMemory) {
        for event in self.events.iter() {

            let blame: String;

            match &event.event {
                screeps::EventType::Attack(attack_event) => {
                    match attack_event.attack_type {
                        screeps::AttackType::Melee => {
                            // Check if the creep was attacked by an enemy player
                            if let Some(player) = creep_attacked(&attack_event.target, &mut memory.room_cache) {
                                blame = player;
                            }

                            continue;
                        },
                        screeps::AttackType::Ranged => {
                            // Check if the creep was attacked by an enemy player
                            if let Some(player) = creep_attacked(&attack_event.target, &mut memory.room_cache) {
                                blame = player;
                            }

                            continue;
                        },
                        screeps::AttackType::RangedMass => {
                            // Check if the creep was attacked by an enemy player
                            if let Some(player) = creep_attacked(&attack_event.target, &mut memory.room_cache) {
                                blame = player;
                            }

                            continue;
                        },
                        screeps::AttackType::Dismantle => todo!(),
                        screeps::AttackType::HitBack => todo!(),
                        screeps::AttackType::Nuke => todo!(),
                    }
                },
                screeps::EventType::AttackController => todo!(),
                _ => {}
            }
        }
    }
}