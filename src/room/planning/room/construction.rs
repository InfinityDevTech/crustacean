use log::info;
use screeps::{HasId, HasPosition, Position, Room, StructureType};

use crate::{room::cache::tick_cache::CachedRoom, traits::position::PositionExtensions};

fn find_pos_most_accessible(
    room: &Room,
    room_cache: &CachedRoom,
    start_pos: &Position,
    range: u8,
) -> Option<Position> {
    let accessible_positions = start_pos.get_accessible_positions_around(range);

    let to_score = room_cache.storage_center.unwrap();
    let to_score_pos = Position::new(to_score.x, to_score.y, room.name());

    let mut closest_distance = u32::MAX;
    let mut closest = None;

    // This ranks positions around something that we want, and returns the position that
    // is the most accessible, while also being the closest.
    for accessible_pos in accessible_positions {
        let mut distance = accessible_pos.get_range_to(to_score_pos);

        let mut other_accessible_positions = 0;
        for pos in accessible_pos.get_accessible_positions_around(1) {
            if pos.is_near_to(accessible_pos) && start_pos.get_range_to(pos) <= range as u32 {
                other_accessible_positions += 1;
            }
        }

        if distance > other_accessible_positions {
            distance -= other_accessible_positions;
        }

        if distance < closest_distance {
            closest_distance = distance;
            closest = Some(accessible_pos);
        }
    }

    closest
}

pub fn plan_containers_and_links(room: &Room, room_cache: &CachedRoom) {
    let mut links_placed = 0;

    let max_links = match room_cache.rcl {
        5 => 1,
        6 => 1,
        7 => 2,
        8 => 3,
        _ => 0,
    };

    if let Some(controller) = &room_cache.structures.controller {
        if controller.container.is_some() || controller.link.is_some() {
            if controller.link.is_some() {
                links_placed += 1;
            }
        } else {
            let container_pos =
                find_pos_most_accessible(room, room_cache, &controller.controller.pos(), 2);
            let link_pos =
                find_pos_most_accessible(room, room_cache, &controller.controller.pos(), 3);

            if let Some(container_pos) = container_pos {
                let _ = room.create_construction_site(
                    container_pos.x().u8(),
                    container_pos.y().u8(),
                    StructureType::Container,
                    None,
                );
            }

            if let Some(link_pos) = link_pos {
                if links_placed < max_links {
                    links_placed += 1;
                    let _ = room.create_construction_site(
                        link_pos.x().u8(),
                        link_pos.y().u8(),
                        StructureType::Link,
                        None,
                    );
                }
            }
        }
    }

    for source in &room_cache.resources.sources {
        if source.link.is_some() {
            links_placed += 1;
        }
    }

    for source in &room_cache.resources.sources {
        if source.container.is_some() && source.link.is_some() {
            continue;
        }

        let container_pos = find_pos_most_accessible(room, room_cache, &source.source.pos(), 1);
        let link_pos = find_pos_most_accessible(room, room_cache, &source.source.pos(), 2);

        if let Some(container_pos) = container_pos {
            let _ = room.create_construction_site(
                container_pos.x().u8(),
                container_pos.y().u8(),
                StructureType::Container,
                None,
            );
        }

        if let Some(link_pos) = link_pos {
            if links_placed >= max_links {
                continue;
            }

            links_placed += 1;
            let _ = room.create_construction_site(
                link_pos.x().u8(),
                link_pos.y().u8(),
                StructureType::Link,
                None,
            );
        }
    }
}

pub fn get_containers() -> Vec<(i8, i8, StructureType)> {
    vec![
        (-2, -1, StructureType::Container),
        (2, -1, StructureType::Container),
    ]
}

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
        (0, -1, StructureType::Link),
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
        (3, 2, StructureType::Extension),
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
