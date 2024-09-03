// If I set alliance tag to null, I dont want to to be added lol
#![allow(clippy::comparison_to_empty)]

use std::{collections::HashMap, f32::consts::E};

use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{
    constants, game, pathfinder::SearchOptions, BodyPart, LocalCostMatrix,
    OwnedStructureProperties, Part, Position, RectStyle, ResourceType, RoomCoordinate, RoomName,
    RoomXY, Source, Store, Terrain,
};

use crate::{
    config, heap,
    memory::Role,
    movement::move_target::MoveTarget,
    room::cache::{hauling::HaulingPriority, CachedRoom, RoomCache},
    traits::room::RoomType,
};

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_my_username() -> String {
    let mut heap_username = heap().my_username.lock().unwrap();
    if !heap_username.is_empty() {
        return heap_username.clone();
    }

    if let Some(first_creep) = game::creeps().values().next() {
        let user = first_creep.owner().username().to_string();

        heap_username.clone_from(&user);
        return first_creep.owner().username().to_string();
    }

    for room in game::rooms().values() {
        if room.controller().is_some()
            && room.controller().unwrap().my()
            && room.controller().is_some()
            && room.controller().unwrap().my()
        {
            let user = room
                .controller()
                .unwrap()
                .owner()
                .unwrap()
                .username()
                .to_string();

            heap_username.clone_from(&user);
            return user;
        }
    }

    panic!("Unable to determine my username!");
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_room_sign(remote: bool) -> String {
    let alliance_tag = config::ALLIANCE_TAG;

    let to_use = if remote {
        config::REMOTE_SIGNS.to_vec()
    } else {
        config::ROOM_SIGNS.to_vec()
    };

    let mut seedable = StdRng::seed_from_u64(game::time().into());
    let sign = to_use[seedable.gen_range(0..to_use.len())];

    if alliance_tag != "" {
        return format!("{} {}", alliance_tag, sign);
    }

    sign.to_string()
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn room_type(name: &RoomName) -> RoomType {
    let x_coord = name.x_coord();
    let x_mod = if x_coord < 0 {
        (x_coord.abs() - 1) % 10
    } else {
        x_coord % 10
    };

    let y_coord = name.y_coord();
    let y_mod = if y_coord < 0 {
        (y_coord.abs() - 1) % 10
    } else {
        y_coord % 10
    };

    if x_mod == 0 && y_mod == 0 {
        return RoomType::Intersection;
    } else if x_mod == 0 || y_mod == 0 {
        return RoomType::Highway;
    } else if x_mod == 5 && y_mod == 5 {
        return RoomType::Center;
    } else if (4..=6).contains(&x_mod) && (4..=6).contains(&y_mod) {
        return RoomType::SourceKeeper;
    }

    RoomType::Normal
}
/// Scale the hauling priority based on the amount of resources in the target.
/// Capacity: The total capacity of the target
/// Amount: The amount of resources in the target
/// Priority: The priority of the task
/// Reverse: Get priority based off of how empty the container is
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn scale_haul_priority(
    capacity: u32,
    amount: u32,
    priority: HaulingPriority,
    reverse: bool,
) -> f32 {
    let priority = (priority as u32) as f32;
    let capacity = capacity as f32;
    let amount = amount as f32;

    if capacity == 0.0 {
        return 0.0;
    }

    if reverse {
        return (1.0 - amount / capacity) * priority;
    }

    (amount / capacity) * priority
}

/// Convert a role to its respective string
/// **Example:** Miner **=** sm
/// **Example:** Hauler **=** mb
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn role_to_name(role: Role) -> String {
    let data = match role {
        Role::Harvester => "sm",
        Role::MineralMiner => "mm",
        Role::Hauler => "mb",
        Role::Repairer => "rb",
        Role::BaseHauler => "bh",
        Role::StorageSitter => "ss",
        Role::Upgrader => "ud",
        Role::Builder => "bd",
        Role::Scout => "fg",
        Role::FastFiller => "ff",
        Role::Bulldozer => "sa",
        Role::GiftBasket => "gb",
        Role::RemoteHarvester => "rm",
        Role::PhysicalObserver => "po",
        Role::Unclaimer => "uc",
        Role::Recycler => "rc",
        Role::ExpansionBuilder => "eb",

        Role::Claimer => "cl",
        Role::Reserver => "rs",
        Role::RemoteDefender => "rd",
        Role::InvaderCoreCleaner => "ic",
        Role::InvaderDuoAttacker => "ia",
        Role::InvaderDuoHealer => "ih",
    };
    data.to_string()
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn source_max_parts(source: &Source) -> u8 {
    let max_energy = source.energy_capacity();

    // Each work part equates to 2 energy per tick
    // Each source refills energy every 300 ticks.
    let max_work_needed = (max_energy / 600) + 1;

    max_work_needed.clamp(0, u8::MAX as u32) as u8
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn under_storage_gate(room_cache: &CachedRoom, gate: f32) -> bool {
    let storage_gate = room_cache.storage_status.wanted_energy as f32 * gate;

    if let Some(storage) = &room_cache.structures.storage {
        if storage
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            < storage_gate.round() as u32
        {
            return true;
        }
    }

    false
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn calc_terminal_cost(amount: u32, source: &RoomName, dest: &RoomName) -> u32 {
    let dist = calc_room_distance(source, dest, true);

    (amount as f32 * (1.0 - E.powf(-dist as f32 / 30.0))).ceil() as u32
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn calc_room_distance(source: &RoomName, dest: &RoomName, continous: bool) -> i32 {
    let mut dx = (source.x_coord() - dest.x_coord()).abs();
    let mut dy = (source.y_coord() - dest.y_coord()).abs();

    if continous {
        let world_size = game::map::get_world_size() as i32;

        dx = (world_size - dx).min(dx);
        dy = (world_size - dy).min(dy);
    }

    dx.max(dy)
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn contains_other_than(store: &Store, resource: ResourceType) -> bool {
    let total_capacity = store.get_used_capacity(None);
    let total_amount = store.get_used_capacity(Some(resource));

    if store.get_used_capacity(None) == 0 {
        return false;
    }

    total_capacity != total_amount
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn store_to_hashmap(store: &Store) -> HashMap<ResourceType, u32> {
    let mut map = HashMap::new();

    for resource in constants::RESOURCES_ALL.iter() {
        let amount = store.get_used_capacity(Some(*resource));
        if amount > 0 {
            map.insert(*resource, amount);
        }
    }

    map
}

/// Convert a string to its respective role
/// **Example:** sm **=** Miner
/// **Example:** mb **=** Hauler
//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn name_to_role(name: &str) -> Option<Role> {
    let role_tag = name.split("-").next().unwrap();
    match role_tag {
        "sm" => Some(Role::Harvester),
        "mb" => Some(Role::Hauler),
        "mm" => Some(Role::MineralMiner),
        "rb" => Some(Role::Repairer),
        "bh" => Some(Role::BaseHauler),
        "ss" => Some(Role::StorageSitter),
        "ud" => Some(Role::Upgrader),
        "bd" => Some(Role::Builder),
        "fg" => Some(Role::Scout),
        "ff" => Some(Role::FastFiller),
        "sa" => Some(Role::Bulldozer),
        "gb" => Some(Role::GiftBasket),
        "rm" => Some(Role::RemoteHarvester),
        "uc" => Some(Role::Unclaimer),
        "rc" => Some(Role::Recycler),
        "po" => Some(Role::PhysicalObserver),
        "cl" => Some(Role::Claimer),
        "rs" => Some(Role::Reserver),
        "rd" => Some(Role::RemoteDefender),
        "ic" => Some(Role::InvaderCoreCleaner),
        "ia" => Some(Role::InvaderDuoAttacker),
        "ih" => Some(Role::InvaderDuoHealer),

        "eb" => Some(Role::ExpansionBuilder),
        _ => None,
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_closest_owned_room(
    target_room: &RoomName,
    cache: &RoomCache,
    min_rcl: Option<u8>,
) -> Option<RoomName> {
    let mut closest_room = None;
    let mut closest_distance = 0;

    if cache.my_rooms.contains(target_room) {
        if let Some(min_rcl) = min_rcl {
            if cache.rooms.contains_key(target_room) && cache.rooms.get(target_room).unwrap().rcl >= min_rcl {
                return Some(*target_room);
            }
        } else {
            return Some(*target_room);
        }
    }

    let coord = RoomCoordinate::new(25).unwrap();
    let target_position = Position::new(coord, coord, *target_room);

    for room in &cache.my_rooms {
        let position = Position::new(coord, coord, *room);

        let distance = target_position.get_range_to(position);

        if let Some(min_rcl) = min_rcl {
            let room = game::rooms().get(*room).unwrap();
            if room.controller().unwrap().level() < min_rcl {
                continue;
            }
        }

        if closest_room.is_none() || distance < closest_distance {
            closest_room = Some(*room);
            closest_distance = distance;
        }
    }

    closest_room
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_unique_id() -> u128 {
    let mut lock = heap().unique_id.lock().unwrap();
    let id = *lock;
    *lock += 1;
    id
}

pub fn moving_average(old_value: f64, new_value: f64, alpha: f64) -> f64 {
    (alpha * new_value) + (1.0 - alpha) * old_value
}

pub fn get_part_count(parts: &Vec<BodyPart>, part_type: Option<Part>) -> u8 {
    let mut count = 0;

    for part in parts {
        if let Some(part_type) = part_type {
            if part.part() == part_type {
                count += 1;
            }
        } else {
            count += 1;
        }
    }

    count
}

pub fn get_pathfind_distance(pos: Position, target: Position) -> u32 {
    let path = MoveTarget { pos, range: 1 }.pathfind(target, Some(SearchOptions::default()));

    if path.incomplete() {
        return pos.get_range_to(target) * 2;
    }

    path.path().len() as u32
}

pub fn calculate_swamp_percentage(room_name: &RoomName) -> f64 {
    let terrain = game::map::get_room_terrain(*room_name).unwrap();

    let mut total_tiles = 0;
    let mut swamp_tiles = 0;

    for x in 0..50 {
        for y in 0..50 {
            let tile = terrain.get(x, y);

            if tile == Terrain::Wall {
                continue;
            }

            total_tiles += 1;

            if tile == Terrain::Swamp {
                swamp_tiles += 1;
            }
        }
    }

    (swamp_tiles as f64 / total_tiles as f64) * 100.0
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_rampart_repair_rcl(rcl: u8) -> u32 {
    match rcl {
        1 => 500,
        2 => 2000,
        3 => 10000,
        4 => 100000,
        5 => 500000,
        6 => 1000000,
        7 => 10000000,
        8 => 10000000,
        _ => 0,
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn get_body_cost(parts: &Vec<Part>) -> u32 {
    let mut cost = 0;

    for part in parts {
        cost += part.cost();
    }

    cost
}

pub fn new_xy(x: u8, y: u8) -> RoomXY {
    RoomXY::new(
        RoomCoordinate::new(x.clamp(0, 49)).unwrap(),
        RoomCoordinate::new(y.clamp(0, 49)).unwrap(),
    )
}

pub fn distance_transform(
    room_name: &RoomName,
    input_cm: Option<LocalCostMatrix>,
    visual: bool,
) -> LocalCostMatrix {
    let mut cm = if let Some(input) = input_cm {
        input
    } else {
        let terrain = game::map::get_room_terrain(*room_name).unwrap();

        let mut cm = LocalCostMatrix::new();

        for x in 0..50 {
            for y in 0..50 {
                let score = if terrain.get(x, y) == Terrain::Wall {
                    0
                } else {
                    255
                };

                cm.set(new_xy(x, y), score)
            }
        }

        cm
    };

    let mut top: u8;
    let mut left: u8;
    let mut top_left: u8;
    let mut top_right: u8;
    let mut bottom_left: u8;

    for x in 0..50 {
        for y in 0..50 {
            top = cm.get(new_xy(x, y - 1));
            left = cm.get(new_xy(x - 1, y));
            top_left = cm.get(new_xy(x - 1, y - 1));
            top_right = cm.get(new_xy(x + 1, y - 1));
            bottom_left = cm.get(new_xy(x - 1, y + 1));

            let coord = new_xy(x, y);

            let num1 = top.min(left).min(top_left).min(top_right).min(bottom_left) + 1;
            let num2 = cm.get(coord);
            cm.set(coord, num1.min(num2) as u8);
        }
    }

    let mut bottom;
    let mut right;
    let mut bottom_right;

    for x in (0..50).rev() {
        for y in (0..50).rev() {
            bottom = cm.get(new_xy(x, y + 1));
            right = cm.get(new_xy(x + 1, y));
            bottom_right = cm.get(new_xy(x + 1, y + 1));
            top_right = cm.get(new_xy(x + 1, y - 1));
            bottom_left = cm.get(new_xy(x - 1, y + 1));

            let num1 = bottom
                .min(right)
                .min(bottom_right)
                .min(top_right)
                .min(bottom_left)
                + 1;
            let num2 = cm.get(new_xy(x, y));
            cm.set(new_xy(x, y), num1.min(num2) as u8);
        }
    }

    if visual {
        if let Some(game_room) = game::rooms().get(*room_name) {
            let vis = game_room.visual();
            for x in 1..49 {
                for y in 1..49 {
                    let score = cm.get(new_xy(x, y));

                    if score == 255 {
                        continue;
                    }

                    vis.rect(
                        x as f32 - 0.5,
                        y as f32 - 0.5,
                        1.0,
                        1.0,
                        Some(
                            RectStyle::default().fill(
                                &format!(
                                    "hsl({}, 100%, 60%)",
                                    200 * cm.get(new_xy(x as u8, y as u8)) / 10
                                )
                                .as_str(),
                            ),
                        ),
                    );
                    vis.text(
                        x as f32,
                        y as f32,
                        cm.get(new_xy(x, y)).to_string(),
                        Some(Default::default()),
                    );
                }
            }
        }
    }

    cm
}
