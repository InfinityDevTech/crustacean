use std::vec;

use screeps::{game, Creep, HasPosition, Position};

#[derive(Debug, Clone)]
pub struct HeapCreep {
    pub health: u32,
    // Stuck Detection
    pub previous_positions: Vec<Position>,
    pub stuck: bool,
    pub last_checked: u32,
}

#[derive(PartialEq)]
pub enum HealthChangeType {
    Damage,
    Heal,
    None,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl HeapCreep {
    pub fn new(creep: &Creep) -> HeapCreep {
        HeapCreep {
            health: creep.hits(),
            // Stuck Detection
            previous_positions: vec![creep.pos()],
            stuck: false,
            last_checked: 0,
        }
    }

    pub fn update_position(&mut self, creep: &Creep) {
        if self.last_checked == game::time() {
            return;
        }

        if self.previous_positions.len() >= 10 {
            self.previous_positions.truncate(9);
        }

        self.previous_positions.insert(0, creep.pos());
        self.last_checked = game::time();

        self.calculate_position_uniqueness();
    }

    pub fn calculate_position_uniqueness(&mut self) {
        let mut unique = Vec::new();

        if self.previous_positions.len() < 7 {
            self.stuck = false;
            return;
        }

        for pos in &self.previous_positions {
            if !unique.contains(pos) {
                unique.push(*pos);
            }
        }

        self.stuck = unique.len() < 5;
    }

    pub fn get_health_change(&mut self, creep: &Creep) -> HealthChangeType {
        let change_type = match creep.hits() {
            h if h < self.health => HealthChangeType::Damage,
            h if h > self.health => HealthChangeType::Heal,
            _ => HealthChangeType::None,
        };

        self.health = creep.hits();

        change_type
    }
}