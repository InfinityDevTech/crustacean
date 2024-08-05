use screeps::{game, Creep, HasPosition, Position};

#[derive(Debug, Clone)]
pub struct HeapCreep {
    pub health: u32,
    // Stuck Detection
    pub previous_position: Position,
    pub stuck_time: u8,
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
            previous_position: creep.pos(),
            stuck_time: 0,
            last_checked: 0,
        }
    }

    pub fn update_position(&mut self, creep: &Creep) {
        if self.last_checked == game::time() {
            return;
        }

        if creep.pos() == self.previous_position {
            self.stuck_time += 1;
        } else {
            self.stuck_time = 0;
        }

        self.last_checked = game::time();
        self.previous_position = creep.pos();
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