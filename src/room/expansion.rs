use std::{collections::HashMap, default, intrinsics::sqrtf32};

use js_sys::Math::sqrt;
use log::info;
use screeps::{
    game::{
        self,
        map::{RoomStatus, RoomStatusResult},
    },
    pathfinder::SearchOptions,
    MapTextStyle, MapVisual, Position, ResourceType, RoomCoordinate, RoomName, RoomXY,
};
use serde::{Deserialize, Serialize};

use crate::{
    config,
    constants::ROOM_SIZE,
    memory::ScreepsMemory,
    movement::move_target::{MoveOptions, MoveTarget},
    traits::{
        position::RoomXYExtensions,
        room::{RoomNameExtensions, RoomType},
    },
    utils,
};

use super::{
    cache::RoomCache,
    planning::room::{self, remotes::rank_remote_room},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ExpansionStatus {
    #[default]
    FindingCapable,
    ScoringRooms,
    FinalTouches,
    Expanding,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ExpansionMemory {
    pub start_time: u32,
    pub current_status: ExpansionStatus,

    pub potential_rooms: Vec<RoomName>,
    pub rooms_can_fit: HashMap<RoomName, bool>,
    pub scored_rooms: HashMap<RoomName, f64>,
}

pub fn can_expand(memory: &ScreepsMemory) -> bool {
    let room_count = memory.rooms.len();
    let gcl = game::gcl::level() as usize;
    let claim_goals = memory.goals.room_claim.len();

    room_count + claim_goals < gcl
}

pub fn attempt_expansion(memory: &mut ScreepsMemory, cache: &RoomCache) {
    let mut expansion_memory = if memory.expansion.is_none() {
        ExpansionMemory::default()
    } else {
        memory.expansion.as_ref().unwrap().clone()
    };

    info!("[EXPANSION] Expansion status: {:?}", expansion_memory.current_status);

    match expansion_memory.current_status {
        ExpansionStatus::FindingCapable => {
            let (expandable_rooms, total_rooms, unscouted_rooms) =
                find_expandable_rooms(memory, cache);

            let percent_unscouted = (unscouted_rooms as f32 / total_rooms as f32) * 100.0;

            if percent_unscouted >= 50.0 {
                info!(
                    "[EXPANSION] We havent scouted around {:.2}% of rooms, pausing expansion until we scout 50%.",
                    percent_unscouted
                );
                return;
            }

            expansion_memory.current_status = ExpansionStatus::ScoringRooms;
            expansion_memory.potential_rooms = expandable_rooms;
        }
        ExpansionStatus::ScoringRooms => {
            let mut scored_rooms = expansion_memory.scored_rooms.clone();
            let potential_rooms = expansion_memory.potential_rooms.clone();

            info!("[EXPANSION] Scoring {} rooms.", potential_rooms.len());

            let mut status = ExpansionStatus::ScoringRooms;
            let mut i = 0;

            let mut unscored_rooms = Vec::new();
            for room in &potential_rooms {
                if !scored_rooms.contains_key(room) {
                    unscored_rooms.push(*room);
                }
            }

            let needed_minerals = get_needed_minerals(memory, cache);

            for room in &unscored_rooms {
                if game::cpu::get_used() > game::cpu::tick_limit() - 100.0 {
                    break;
                }

                info!("[EXPANSION] Attempting to score room: {}", room);

                let score = score_room(room, needed_minerals.clone(), memory, cache);
                scored_rooms.insert(*room, score);

                i += 1;
            }

            if i == unscored_rooms.len() {
                status = ExpansionStatus::FinalTouches;
            }

            expansion_memory.current_status = status;
            expansion_memory.scored_rooms = scored_rooms;
        }
        ExpansionStatus::FinalTouches => {
            let mut scores = expansion_memory.scored_rooms.clone();
            let mut fittable = expansion_memory.rooms_can_fit.clone();
            let mut status = ExpansionStatus::FinalTouches;

            info!("[EXPANSION] Final touches on expansions.");

            let mut unchecked_rooms = Vec::new();
            for (name, score) in &scores {
                if !fittable.contains_key(name) {
                    unchecked_rooms.push(*name);
                }
            }

            let mut i = 0;
            for room in &unchecked_rooms {
                if game::cpu::get_used() > game::cpu::tick_limit() - 100.0 {
                    break;
                }

                let (pos, can_fit) = can_fit_base(room);
                fittable.insert(*room, can_fit);
                i += 1;
            }

            if i == unchecked_rooms.len() {
                status = ExpansionStatus::Expanding;
            }

            if scores.is_empty() {
                info!("[EXPANSION] No scorable rooms found, waiting to expand.");
                expansion_memory.current_status = ExpansionStatus::FindingCapable;
                return;
            }
            expansion_memory.rooms_can_fit = fittable;
            expansion_memory.current_status = status;
        }
        ExpansionStatus::Expanding => {
            let mut scores = expansion_memory.scored_rooms.clone();
            let mut fittable = expansion_memory.rooms_can_fit.clone();

            let mut sorted_scores = scores.iter().collect::<Vec<_>>();
            sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            for (room_name, score) in sorted_scores {
                if fittable.contains_key(room_name) && *fittable.get(room_name).unwrap() {
                    info!(
                        "[EXPANSION] Expanding to room: {} with score: {}",
                        room_name, score
                    );
                    //room::claim::claim_room(room_name, memory, cache);
                }
            }
        }
    }

    memory.expansion = Some(expansion_memory);
}

pub fn find_expandable_rooms(
    memory: &ScreepsMemory,
    cache: &RoomCache,
) -> (Vec<RoomName>, u32, u32) {
    let mut scorable_rooms = Vec::new();
    let mut non_scouted_count = 0;
    let mut total_room_count = 0;

    let mut active_rooms = Vec::new();
    let mut available_rooms = Vec::new();

    for (room_name, room_memory) in &memory.rooms {
        available_rooms.append(&mut room_name.get_adjacent(config::MAX_CLAIM_DISTANCE as i32));
    }

    for possible_name in available_rooms {
        let closest_room = utils::find_closest_owned_room(&possible_name, cache, None);

        if let Some(closest_room) = closest_room {
            let dist = utils::calc_room_distance(&closest_room, &possible_name, true) as u32;

            if !(config::MIN_CLAIM_DISTANCE..=config::MAX_CLAIM_DISTANCE).contains(&dist) {
                continue;
            }

            if !memory.rooms.contains_key(&possible_name) {
                if let Some(scouting_data) = memory.scouted_rooms.get(&possible_name) {
                    if scouting_data.room_type == RoomType::Normal
                        && scouting_data.sources.is_some()
                        && scouting_data.mineral.is_some()
                        && scouting_data.owner.is_none()
                        && scouting_data.reserved.is_none()
                    {
                        scorable_rooms.push(possible_name);
                    }
                } else {
                    non_scouted_count += 1;
                }

                total_room_count += 1;
            }
        }
    }

    for room in scorable_rooms {
        if let Some(status) = game::map::get_room_status(room) {
            if status.status() == RoomStatus::Normal {
                active_rooms.push(room);
            }
        }
    }

    (active_rooms, total_room_count, non_scouted_count)
}

pub fn get_needed_minerals(memory: &ScreepsMemory, cache: &RoomCache) -> Vec<ResourceType> {
    let mut current_minerals = Vec::new();

    for (room_name, room_memory) in &memory.rooms {
        if let Some(cache) = cache.rooms.get(&room_name) {
            if let Some(mineral) = &cache.resources.mineral {
                current_minerals.push(mineral.mineral_type());
            }
        }
    }

    let mut has_basics = current_minerals.contains(&ResourceType::Hydrogen)
        && current_minerals.contains(&ResourceType::Oxygen);
    let mut has_t1_boosts = current_minerals.contains(&ResourceType::Zynthium)
        && current_minerals.contains(&ResourceType::Keanium)
        && current_minerals.contains(&ResourceType::Utrium)
        && current_minerals.contains(&ResourceType::Lemergium);
    let mut has_t3 = has_basics && has_t1_boosts;

    let mut needed_minerals = Vec::new();

    // We want to prioritize T1 minerals.
    // If we don't have the basics, we need to get those first.
    // That way, we dont grab an X room if we don't have the basics.
    if !has_basics {
        needed_minerals.push(ResourceType::Hydrogen);
        needed_minerals.push(ResourceType::Oxygen);

        if !has_t1_boosts {
            needed_minerals.push(ResourceType::Zynthium);
            needed_minerals.push(ResourceType::Keanium);
            needed_minerals.push(ResourceType::Utrium);
            needed_minerals.push(ResourceType::Lemergium);
        }

        return needed_minerals;
    }

    if !has_t1_boosts {
        needed_minerals.push(ResourceType::Zynthium);
        needed_minerals.push(ResourceType::Keanium);
        needed_minerals.push(ResourceType::Utrium);
        needed_minerals.push(ResourceType::Lemergium);

        return needed_minerals;
    }

    if !has_t3 {
        needed_minerals.push(ResourceType::Catalyst);
    }

    needed_minerals
}

pub fn score_room(
    room_name: &RoomName,
    needed_minerals: Vec<ResourceType>,
    memory: &mut ScreepsMemory,
    cache: &RoomCache,
) -> f64 {
    let mut score = 0.0;
    let room_status = game::map::get_room_status(*room_name);

    if !memory.scouted_rooms.contains_key(room_name)
        || room_status.is_none()
        || room_status.unwrap().status() != RoomStatus::Normal
    {
        return score;
    }

    let scouting_data = memory.scouted_rooms.get(room_name).unwrap();

    if scouting_data.room_type != RoomType::Normal
        || scouting_data.sources.is_none()
        || scouting_data.mineral.is_none()
    {
        return score;
    }

    // Actual Scoring.

    // Sources, if it has more than 1, add 100 points per source.
    score += (scouting_data.sources.as_ref().unwrap().len() as f64 - 1.0) * 100.0;
    // If we want the mineral, add 50 points.
    if needed_minerals.contains(&scouting_data.mineral.unwrap()) {
        score += 50.0;
    }
    // Get remotes score.
    score += score_remotes(room_name, memory) * 1.5;

    score
}

// TODO:
// Remove this check once we stop using bunkers.
pub fn can_fit_base(room_name: &RoomName) -> (RoomXY, bool) {
    let distance_transform = utils::distance_transform(room_name, false);

    let mut highest_position = 0;
    let mut highest_xy = utils::new_xy(0, 0);

    for x in 0..50 {
        for y in 0..50 {
            let xy = utils::new_xy(x, y);

            if distance_transform.get(xy) > highest_position {
                highest_xy = xy;
                highest_position = distance_transform.get(xy);
            }
        }
    }

    (highest_xy, highest_position >= 5)
}

pub fn score_remotes(room_name: &RoomName, memory: &ScreepsMemory) -> f64 {
    let potential_remotes = room_name.get_adjacent(2 as i32);
    let mut score = 0.0;

    let mut paths = Vec::new();

    let room_pos = Position::new(
        RoomCoordinate::new(25).unwrap(),
        RoomCoordinate::new(25).unwrap(),
        *room_name,
    );

    for remote in potential_remotes {
        if let Some(scouting_data) = memory.scouted_rooms.get(&remote) {
            if let Some(sources) = &scouting_data.sources {
                for source in sources {
                    let opts = SearchOptions::default().max_ops(20000);
                    let target = MoveTarget {
                        pos: room_pos,
                        range: 1,
                    }
                    .pathfind(source.pos.as_position(&remote), Some(opts));

                    if !target.incomplete() {
                        paths.push(target.path().len());
                    }
                }
            }
        }
    }
    paths.sort();
    paths.reverse();

    let max = (ROOM_SIZE * 2) as f64;
    let mut index = 0;

    for path in paths.clone() {
        let dist = (max - (path as f64).min(max - 1.0)).sqrt();
        let weight = (index / paths.len()) as f64 * 0.5 + 0.5;

        score += dist * weight;

        index += 1;
    }

    score
}
