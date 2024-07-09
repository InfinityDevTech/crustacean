// If I set alliance tag to null, I dont want to to be added lol
#![allow(clippy::comparison_to_empty)]

use rand::{rngs::StdRng, Rng, SeedableRng};
use screeps::{game, OwnedStructureProperties, Part, Position, RoomCoordinate, RoomName};

use crate::{config, heap, memory::Role, room::cache::tick_cache::{hauling::HaulingPriority, RoomCache}, traits::room::RoomType};

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
pub fn get_room_sign() -> String {
    let alliance_tag = config::ALLIANCE_TAG;

    let mut seedable = StdRng::seed_from_u64(game::time().into());
    let sign = config::ROOM_SIGNS[seedable.gen_range(0..config::ROOM_SIGNS.len())];

    if alliance_tag != "" {
        return format!("{} {}", alliance_tag, sign);
    }

    sign.to_string()
}

pub fn room_type(name: &RoomName) -> RoomType {
    let room_x = name.x_coord();
    let room_y = name.y_coord();

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
    if (5 - ew).abs() <= 1 && (5 - ns).abs() <= 1 {
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
        Role::StorageHauler => "sh",
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

        Role::Claimer => "cl",
        Role::Reserver => "rs",
        Role::RemoteDefender => "rd",
        Role::InvaderCleaner => "ic",
    };
    data.to_string()
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
        "sh" => { Some(Role::StorageHauler) },
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
        "ic" => { Some(Role::InvaderCleaner) },
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