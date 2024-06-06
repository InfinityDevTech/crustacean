use screeps::StructureType;

pub fn get_rcl_2_plan() -> Vec<(i8, i8, StructureType)> {
    vec![
        (-2, -1, StructureType::Container),
        (2, -1, StructureType::Container),

        (-1, 0, StructureType::Extension),
        (1, 0, StructureType::Extension),
        (2, 0, StructureType::Extension),
        (-2, 0, StructureType::Extension),
        (0, -2, StructureType::Extension),
    ]
}

pub fn get_rcl_3_plan() -> Vec<(i8, i8, StructureType)> {
    vec![
        (5, 2, StructureType::Tower),

        (-2, -2, StructureType::Extension),
        (2, -2, StructureType::Extension),
        (4, 3, StructureType::Extension),
        (4, 4, StructureType::Extension),
        (3, 4, StructureType::Extension),
    ]
}

pub fn get_rcl_4_plan() -> Vec<(i8, i8, StructureType)> {
    vec![
        (0, 4, StructureType::Storage),

        (4, 6, StructureType::Extension),
        (5, 5, StructureType::Extension),
        (6, 4, StructureType::Extension),
        (6, 3, StructureType::Extension),
        (6, 3, StructureType::Extension),
        (-5, 5, StructureType::Extension),
        (-6, 4, StructureType::Extension),
        (-6, 3, StructureType::Extension),
        (-6, 1, StructureType::Extension),
        (-1, 2, StructureType::Extension),
    ]
}

pub fn get_rcl_5_plan() -> Vec<(i8, i8, StructureType)> {
    vec![
        (-5, 2, StructureType::Tower),
        (0, -1, StructureType::Link),
        (0, 2, StructureType::Link),

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

    ]
}

pub fn get_rcl_6_plan() -> Vec<(i8, i8, StructureType)> {
    vec![
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

        (2, 2, StructureType::Terminal),
        (-3, 2, StructureType::Lab),
        (-2, 2, StructureType::Lab),
        (-2, 3, StructureType::Lab),
    ]
}

pub fn get_rcl_7_plan() -> Vec<(i8, i8, StructureType)> {
    vec![
        (-1, 3, StructureType::Lab),
        (-1, 4, StructureType::Lab),
        (-2, 5, StructureType::Lab),
        (4, -2, StructureType::Tower),
        (2, 3, StructureType::Factory),
        (-1, -2, StructureType::Spawn),

        (-5, -3, StructureType::Extension),
        (-1, -4, StructureType::Extension),
        (-2, -4, StructureType::Extension),
        (-3, -4, StructureType::Extension),
        (-4, -4, StructureType::Extension),
        (1, -6, StructureType::Extension),
        (2, -6, StructureType::Extension),
        (3, -6, StructureType::Extension),
        (4, -6, StructureType::Extension),
        (5, -5, StructureType::Extension),
    ]
}

pub fn get_rcl_8_plan() -> Vec<(i8, i8, StructureType)> {
    vec![
        (0, 0, StructureType::Spawn),
        (0, -3, StructureType::Observer),
        (0, 3, StructureType::Nuker),
        (1, 4, StructureType::PowerSpawn),
        (-4, -2, StructureType::Tower),
        (2, 5, StructureType::Tower),
        (0, -5, StructureType::Tower),

        (-3, 5, StructureType::Lab),
        (-3, 4, StructureType::Lab),
        (-4, 4, StructureType::Lab),
        (-4, 3, StructureType::Lab),

        (6, -4, StructureType::Extension),
        (6, 1, StructureType::Extension),
        (-1, -6, StructureType::Extension),
        (-2, -6, StructureType::Extension),
        (-3, -6, StructureType::Extension),
        (-4, -6, StructureType::Extension),
        (-5, -5, StructureType::Extension),
        (-6, -4, StructureType::Extension),
        (-4, 6, StructureType::Extension),
        (0, 5, StructureType::Extension),
    ]
}

pub fn get_roads_and_ramparts() -> Vec<(i8, i8, StructureType)> {
    vec![
        // Start fast-filler core
        // Stop fast-filler core

        // Start production Spot
        // Stop production Spot

        // Start Labs
        // Stop labs

        // Start towers

        (5, 2, StructureType::Rampart),
        (-5, 2, StructureType::Rampart),
        (4, -2, StructureType::Rampart),
        (-4, -2, StructureType::Rampart),
        (2, 5, StructureType::Rampart),
        (0, -5, StructureType::Rampart),
        // Stop Towers

        // Extensions

        // Fast-fill Ramparts
        (0, 0, StructureType::Rampart),
        (0, -1, StructureType::Rampart),
        (1, -1, StructureType::Rampart),
        (1, -2, StructureType::Rampart),
        (-1, -1, StructureType::Rampart),
        (-1, -2, StructureType::Rampart),

        // Factory Ramparts
        (0, 2, StructureType::Rampart),
        (0, 3, StructureType::Rampart),
        (0, 4, StructureType::Rampart),
        (1, 4, StructureType::Rampart),
        (1, 3, StructureType::Rampart),
        (2, 2, StructureType::Rampart),
        (2, 3, StructureType::Rampart),

        // Outer Ramparts
        (0, 6, StructureType::Rampart),
        (1, 6, StructureType::Rampart),
        (2, 6, StructureType::Rampart),
        (3, 6, StructureType::Rampart),
        (4, 6, StructureType::Rampart),
        (5, 6, StructureType::Rampart),
        (6, 6, StructureType::Rampart),
        (6, 5, StructureType::Rampart),
        (6, 4, StructureType::Rampart),
        (6, 3, StructureType::Rampart),
        (6, 2, StructureType::Rampart),
        (6, 1, StructureType::Rampart),
        (6, 0, StructureType::Rampart),
        (6, -1, StructureType::Rampart),
        (6, -2, StructureType::Rampart),
        (6, -3, StructureType::Rampart),
        (6, -4, StructureType::Rampart),
        (6, -5, StructureType::Rampart),
        (6, -6, StructureType::Rampart),
        (5, -6, StructureType::Rampart),
        (4, -6, StructureType::Rampart),
        (3, -6, StructureType::Rampart),
        (2, -6, StructureType::Rampart),
        (1, -6, StructureType::Rampart),
        (0, -6, StructureType::Rampart),
        (-1, -6, StructureType::Rampart),
        (-2, -6, StructureType::Rampart),
        (-3, -6, StructureType::Rampart),
        (-4, -6, StructureType::Rampart),
        (-5, -6, StructureType::Rampart),
        (-6, -6, StructureType::Rampart),
        (-6, -5, StructureType::Rampart),
        (-6, -4, StructureType::Rampart),
        (-6, -3, StructureType::Rampart),
        (-6, -2, StructureType::Rampart),
        (-6, -1, StructureType::Rampart),
        (-6, 0, StructureType::Rampart),
        (-6, 1, StructureType::Rampart),
        (-6, 2, StructureType::Rampart),
        (-6, 3, StructureType::Rampart),
        (-6, 4, StructureType::Rampart),
        (-6, 5, StructureType::Rampart),
        (-6, 6, StructureType::Rampart),
        (-1, 6, StructureType::Rampart),
        (-2, 6, StructureType::Rampart),
        (-3, 6, StructureType::Rampart),
        (-4, 6, StructureType::Rampart),
        (-5, 6, StructureType::Rampart),

        // Roads
        (1, -3, StructureType::Road),
        (2, -3, StructureType::Road),
        (3, -2, StructureType::Road),
        (3, -1, StructureType::Road),
        (3, 0, StructureType::Road),
        (3, 1, StructureType::Road),
        (2, 1, StructureType::Road),
        (1, 1, StructureType::Road),
        (0, 1, StructureType::Road),
        (-1, 1, StructureType::Road),
        (1, 2, StructureType::Road),
        (-2, 1, StructureType::Road),
        (-3, 1, StructureType::Road),
        (-3, 0, StructureType::Road),
        (-3, -1, StructureType::Road),
        (-3, -2, StructureType::Road),
        (-2, -3, StructureType::Road),
        (-1, -3, StructureType::Road),
        (0, -4, StructureType::Road),

        (0, 6, StructureType::Road),
        (-1, 6, StructureType::Road),
        (-2, 6, StructureType::Road),
        (-3, 6, StructureType::Road),
        (1, 6, StructureType::Road),
        (2, 6, StructureType::Road),
        (3, 6, StructureType::Road),
        (1, 5, StructureType::Road),
        (-1, 5, StructureType::Road),
        (-2, 4, StructureType::Road),
        (-3, 3, StructureType::Road),
        (-4, 2, StructureType::Road),
        (-5, 1, StructureType::Road),
        (-6, 0, StructureType::Road),
        (-6, -1, StructureType::Road),
        (6, -1, StructureType::Road),
        (-6, -2, StructureType::Road),
        (-6, -3, StructureType::Road),
        (-5, -4, StructureType::Road),
        (-4, -5, StructureType::Road),
        (-3, -5, StructureType::Road),
        (-2, -5, StructureType::Road),
        (-1, -5, StructureType::Road),
        (0, -6, StructureType::Road),
        (1, -5, StructureType::Road),
        (2, -5, StructureType::Road),
        (3, -5, StructureType::Road),
        (4, -5, StructureType::Road),
        (5, -4, StructureType::Road),
        (6, -3, StructureType::Road),
        (6, -2, StructureType::Road),
        (6, 0, StructureType::Road),
        (6, 2, StructureType::Road),
        (5, 3, StructureType::Road),
        (5, 4, StructureType::Road),
        (4, 5, StructureType::Road),
        (3, 5, StructureType::Road),
        (5, 1, StructureType::Road),
        (4, 2, StructureType::Road),
        (2, 4, StructureType::Road),
        (3, 3, StructureType::Road),
        (-6, 2, StructureType::Road),
        (-5, 3, StructureType::Road),
        (-5, 4, StructureType::Road),
        (-4, 5, StructureType::Road),

        (6, 6, StructureType::Road),
        (5, 6, StructureType::Road),
        (6, 5, StructureType::Road),

        (6, -6, StructureType::Road),
        (5, -6, StructureType::Road),
        (6, -5, StructureType::Road),

        (-6, 6, StructureType::Road),
        (-5, 6, StructureType::Road),
        (-6, 5, StructureType::Road),

        (-6, -6, StructureType::Road),
        (-5, -6, StructureType::Road),
        (-6, -5, StructureType::Road),
    ]
}