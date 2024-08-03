// If I set alliance tag to null, I dont want to to be added lol
#![allow(clippy::comparison_to_empty)]

use std::{collections::HashMap, f32::consts::E};

use log::info;
use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{constants, game, OwnedStructureProperties, Part, Position, ResourceType, RoomCoordinate, RoomName, Store};

use crate::{config, heap, memory::Role, room::cache::{hauling::HaulingPriority, RoomCache}, traits::room::{RoomNameExtensions, RoomType}};

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
        if room.controller().is_some() && room.controller().unwrap().my() && room.controller().is_some() && room.controller().unwrap().my() {
            let user = room.controller().unwrap().owner().unwrap().username().to_string();

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
    let (room_x , room_y) = get_proper_coords(name);

    let ew = room_x % 10;
    let ns = room_y % 10;

    if ew == 0 && ns == 0 {
        return RoomType::Intersection
    }
    if ew == 0 || ns == 0 {
        return RoomType::Highway
    }
    if room_x % 5 == 0 && room_y % 5 == 0 {
        return RoomType::Center
    }
    if 5 - ew <= 1 && 5 - ns <= 1 {
        return RoomType::SourceKeeper
    }

    RoomType::Normal
}
/// Scale the hauling priority based on the amount of resources in the target.
/// Capacity: The total capacity of the target
/// Amount: The amount of resources in the target
/// Priority: The priority of the task
/// Reverse: Get priority based off of how empty the container is
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn scale_haul_priority(capacity: u32, amount: u32, priority: HaulingPriority, reverse: bool) -> f32 {
    let priority = (priority as u32) as f32;
    let capacity = capacity as f32;
    let amount = amount as f32;

    if capacity == 0.0 {
        return 0.0;
    }

    if reverse {
        return (1.0 - amount / capacity) * priority
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
        Role::Miner => "md",
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
pub fn get_proper_coords(room: &RoomName) -> (i32, i32) {
    let x_coord = room.x_coord();
    let y_coord = room.y_coord();

    let x_mod = if x_coord < 0 {
        x_coord + 1
    } else {
        x_coord
    };

    let y_mod = if y_coord < 0 {
        y_coord + 1
    } else {
        y_coord
    };

    (x_mod, y_mod)
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
    let total_capacity = store.get_capacity(Some(resource));
    let total_amount = store.get_used_capacity(Some(resource));

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
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn name_to_role(name: &str) -> Option<Role> {
    let role_tag = name.split("-").next().unwrap();
    match role_tag {
        "sm" => { Some(Role::Harvester) },
        "mb" => { Some(Role::Hauler) },
        "md" => { Some(Role::Miner) },
        "rb" => { Some(Role::Repairer) },
        "bh" => { Some(Role::BaseHauler) },
        "ss" => { Some(Role::StorageSitter) },
        "ud" => { Some(Role::Upgrader) },
        "bd" => { Some(Role::Builder) },
        "fg" => { Some(Role::Scout) },
        "ff" => { Some(Role::FastFiller) }
        "sa" => { Some(Role::Bulldozer) },
        "gb" => { Some(Role::GiftBasket) },
        "rm" => { Some(Role::RemoteHarvester) },
        "uc" => { Some(Role::Unclaimer) },
        "rc" => { Some(Role::Recycler) },
        "po" => { Some(Role::PhysicalObserver) },
        "cl" => { Some(Role::Claimer) },
        "rs" => { Some(Role::Reserver) },
        "rd" => { Some(Role::RemoteDefender) },
        "ic" => { Some(Role::InvaderCoreCleaner) },
        "ia" => { Some(Role::InvaderDuoAttacker) },
        "ih" => { Some(Role::InvaderDuoHealer) },

        "eb" => { Some(Role::ExpansionBuilder) }
        _ => { None },
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn find_closest_owned_room(target_room: &RoomName, cache: &RoomCache, min_rcl: Option<u8>) -> Option<RoomName> {
    let mut closest_room = None;
    let mut closest_distance = 0;

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