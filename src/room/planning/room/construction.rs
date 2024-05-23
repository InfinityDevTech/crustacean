use screeps::{Structure, StructureType};

pub fn get_bunker_plan() -> Vec<(i8, i8, StructureType)> {
    vec![
        // Start fast-filler core
        (0, 0, StructureType::Spawn),
        (-1, -2, StructureType::Spawn),
        (1, -2, StructureType::Spawn),

        (0, 1, StructureType::Extension),
        (-1, 0, StructureType::Extension),
        (1, 0, StructureType::Extension),
        (2, 0, StructureType::Extension),
        (-2, 0, StructureType::Extension),
        (-3, -1, StructureType::Extension),
        (3, -1, StructureType::Extension),
        (0, -2, StructureType::Extension),
        (-2, -2, StructureType::Extension),
        (2, -2, StructureType::Extension),

        (0, -1, StructureType::Link),

        (-2, -1, StructureType::Container),
        (2, -1, StructureType::Container),

        (0, -3, StructureType::Observer),
        // Stop fast-filler core

        // Start production Spot
        (0, 4, StructureType::Storage),
        (1, 2, StructureType::Factory),
        (1, 3, StructureType::Link),
        (1, 4, StructureType::PowerSpawn),
        (-1, 2, StructureType::Tower),
        (-1, 3, StructureType::Nuker),
        (-1, 4, StructureType::Terminal),
        // Stop production Spot

        // Start Labs
        (-2, 5, StructureType::Lab),
        (-3, 5, StructureType::Lab),
        (-3, 4, StructureType::Lab),
        (-4, 4, StructureType::Lab),
        (-4, 5, StructureType::Lab),

        (2, 5, StructureType::Lab),
        (3, 5, StructureType::Lab),
        (3, 4, StructureType::Lab),
        (4, 4, StructureType::Lab),
        (4, 5, StructureType::Lab),
        // Stop labs

        // Start towers
        (5, 2, StructureType::Tower),
        (-5, 2, StructureType::Tower),
        (4, -2, StructureType::Tower),
        (-4, -2, StructureType::Tower),
        (0, -5, StructureType::Tower),
        (0, 5, StructureType::Tower),
        // Stop Towers

        // Extensions
        (4, 6, StructureType::Extension),
        (5, 5, StructureType::Extension),
        (6, 4, StructureType::Extension),
        (6, 3, StructureType::Extension),
        (-4, 6, StructureType::Extension),
        (-5, 5, StructureType::Extension),
        (-6, 4, StructureType::Extension),
        (-6, 3, StructureType::Extension),

        (2, 2, StructureType::Extension),
        (2, 3, StructureType::Extension),
        (3, 2, StructureType::Extension),
        (-2, 2, StructureType::Extension),
        (-2, 3, StructureType::Extension),
        (-3, 2, StructureType::Extension),

        (4, 1, StructureType::Extension),
        (4, 0, StructureType::Extension),
        (5, 0, StructureType::Extension),
        (4, -1, StructureType::Extension),
        (5, -1, StructureType::Extension),
        (5, -2, StructureType::Extension),
        (3, -3, StructureType::Extension),
        (4, -3, StructureType::Extension),
        (5, -3, StructureType::Extension),
        (1, -4, StructureType::Extension),
        (2, -4, StructureType::Extension),
        (3, -4, StructureType::Extension),
        (4, -4, StructureType::Extension),

        (-4, 1, StructureType::Extension),
        (-4, 0, StructureType::Extension),
        (-5, 0, StructureType::Extension),
        (-4, -1, StructureType::Extension),
        (-5, -1, StructureType::Extension),
        (-5, -2, StructureType::Extension),
        (-3, -3, StructureType::Extension),
        (-4, -3, StructureType::Extension),
        (-5, -3, StructureType::Extension),
        (-1, -4, StructureType::Extension),
        (-2, -4, StructureType::Extension),
        (-3, -4, StructureType::Extension),
        (-4, -4, StructureType::Extension),

        (0, -6, StructureType::Extension),

        (1, -6, StructureType::Extension),
        (2, -6, StructureType::Extension),
        (3, -6, StructureType::Extension),
        (4, -6, StructureType::Extension),
        (5, -5, StructureType::Extension),
        (6, -4, StructureType::Extension),

        (-1, -6, StructureType::Extension),
        (-2, -6, StructureType::Extension),
        (-3, -6, StructureType::Extension),
        (-4, -6, StructureType::Extension),
        (-5, -5, StructureType::Extension),
        (-6, -4, StructureType::Extension),

        // Fast-fill Ramparts
        (0, 0, StructureType::Rampart),
        (0, -1, StructureType::Rampart),
        (1, -1, StructureType::Rampart),
        (1, -2, StructureType::Rampart),
        (-1, -1, StructureType::Rampart),
        (-1, -2, StructureType::Rampart),

        // Factory Ramparts
        (0, 3, StructureType::Rampart),
        (0, 4, StructureType::Rampart),
        (1, 2, StructureType::Rampart),
        (1, 3, StructureType::Rampart),
        (1, 4, StructureType::Rampart),
        (-1, 2, StructureType::Rampart),
        (-1, 3, StructureType::Rampart),
        (-1, 4, StructureType::Rampart),
    ]
}