use std::collections::HashMap;

use log::info;
use screeps::{
    game::{
        self,
        map::{FindRouteOptions, RoomStatus},
    }, pathfinder::SearchOptions, Position, ResourceType, RoomCoordinate, RoomName, RoomXY
};
use serde::{Deserialize, Serialize};

use crate::{
    config, constants::ROOM_SIZE, goal_memory::RoomClaimGoal, memory::ScreepsMemory, movement::move_target::MoveTarget, traits::{
        position::RoomXYExtensions,
        room::{RoomNameExtensions, RoomType},
    }, utils
};

use super::cache::RoomCache;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ExpansionStatus {
    #[default]
    FindingCapable,
    ScoringRooms,
    FinalTouches,
    Expanding,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExpansionMemory {
    pub start_time: u32,
    pub current_status: ExpansionStatus,
    pub last_chcked: u32,

    pub potential_rooms: Vec<RoomName>,
    pub rooms_can_fit: HashMap<RoomName, bool>,
    pub scored_rooms: HashMap<RoomName, f64>,
}

impl ExpansionMemory {
    pub fn new() -> ExpansionMemory {
        ExpansionMemory {
            start_time: game::time(),
            last_chcked: 0,
            current_status: ExpansionStatus::FindingCapable,
            potential_rooms: Vec::new(),
            rooms_can_fit: HashMap::new(),
            scored_rooms: HashMap::new(),
        }
    }
}

pub fn can_expand(memory: &ScreepsMemory) -> bool {
    let room_count = memory.rooms.len();
    let gcl = game::gcl::level() as usize;
    let claim_goals = memory.goals.room_claim.len();

    // Newbieland has a max of 3 rooms.
    if game::shard::name() == "newbieland" && memory.goals.room_claim.len() + memory.rooms.len() >= 3 {
        return false;
    }

    if !memory.goals.room_claim.is_empty() {
        return false;
    }

    room_count + claim_goals < gcl
}

pub fn attempt_expansion(memory: &mut ScreepsMemory, cache: &RoomCache) {
    let mut expansion_memory = if memory.expansion.is_none() {
        ExpansionMemory::new()
    } else {
        memory.expansion.as_ref().unwrap().clone()
    };

    info!("[EXPANSION] Expansion status: {:?}", expansion_memory.current_status);

    match expansion_memory.current_status {
        ExpansionStatus::FindingCapable => {
            if expansion_memory.last_chcked + 10 > game::time() {
                info!("[EXPANSION] We dont have enough scouting data!");
                return;
            }

            let (expandable_rooms, total_rooms, unscouted_rooms) =
                find_expandable_rooms(memory, cache);

            let percent_unscouted = (unscouted_rooms as f32 / total_rooms as f32) * 100.0;

            // TODO:
            // Improve scouts!!!
            if percent_unscouted >= 70.0 {
                info!(
                    "[EXPANSION] We havent scouted around {:.2}% of rooms, pausing expansion until we scout 70%.",
                    percent_unscouted
                );

                expansion_memory.last_chcked = game::time();
                memory.expansion = Some(expansion_memory);
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

            let mut unscored_rooms = Vec::new();
            for room in &potential_rooms {
                if !scored_rooms.contains_key(room) {
                    unscored_rooms.push(*room);
                }
            }

            let needed_minerals = get_needed_minerals(memory, cache);

            for room in &unscored_rooms {
                info!("[EXPANSION] Scoring room: {}", room);
                if game::cpu::get_used() > game::cpu::tick_limit() - 100.0 {
                    break;
                }

                let score = score_room(room, needed_minerals.clone(), memory, cache);
                scored_rooms.insert(*room, score);
            }

            if unscored_rooms.is_empty() {
                status = ExpansionStatus::FinalTouches;
            }

            expansion_memory.current_status = status;
            expansion_memory.scored_rooms = scored_rooms;
        }
        ExpansionStatus::FinalTouches => {
            let scores = expansion_memory.scored_rooms.clone();
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
            let scores = expansion_memory.scored_rooms.clone();
            let fittable = expansion_memory.rooms_can_fit.clone();

            let mut sorted_scores = scores.iter().collect::<Vec<_>>();
            sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            let mut highest_score = 0.0;
            let mut top_scorer = None;

            for (room_name, score) in sorted_scores {
                if fittable.contains_key(room_name) && *fittable.get(room_name).unwrap() && *score > highest_score {
                    highest_score = *score;
                    top_scorer = Some(*room_name);
                }
            }

            if let Some(top_scorer) = top_scorer {
                info!(
                    "[EXPANSION] Expanding to room: {} with score: {}",
                    top_scorer, highest_score
                );

                if memory.goals.room_claim.contains_key(&top_scorer) {
                    info!("[EXPANSION] Room already claimed, skipping.");
                    memory.expansion = None;

                    return;
                }

                game::notify(format!("[EXPANSION] Attempting to expand to the room: {}", top_scorer).as_str(), None);
                let goal = RoomClaimGoal {
                    claim_target: top_scorer,
                    creeps_assigned: Vec::new(),
                };

                memory.goals.room_claim.insert(top_scorer, goal);
            } else {
                info!("[EXPANSION] Found no suitable rooms to expand to.");
            }
        }
    }

    if !memory.goals.room_claim.is_empty() {
        memory.expansion = None;

        return;
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

            let room_status = game::map::get_room_status(possible_name);
            if room_status.is_none() || room_status.unwrap().status() != RoomStatus::Normal {
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

    let has_basics = current_minerals.contains(&ResourceType::Hydrogen)
        && current_minerals.contains(&ResourceType::Oxygen);
    let has_t1_boosts = current_minerals.contains(&ResourceType::Zynthium)
        && current_minerals.contains(&ResourceType::Keanium)
        && current_minerals.contains(&ResourceType::Utrium)
        && current_minerals.contains(&ResourceType::Lemergium);
    let has_t3 = has_basics && has_t1_boosts;

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

    let (remote_score, remote_source_count) = score_remotes(room_name, memory);
    // Get remotes score.
    score += remote_score * 1.5;
    score += remote_source_count as f64 * 10.0;

    score -= nearby_source_keepers(room_name) * 10.0;
    score -= scan_remote_accessibility(room_name) as f64 * 3.0;
    let swamp_percent = utils::calculate_swamp_percentage(room_name);

    let closest_room = utils::find_closest_owned_room(room_name, cache, None);
    if let Some(closest_room) = closest_room {
        let res = game::map::find_route(*room_name, closest_room, Some(FindRouteOptions::default()));

        if res.is_ok() {
            let path = res.unwrap();
            let path_length = path.len();

            let count = ROOM_SIZE as u32 * path_length as u32;

            if count >= 550 {
                return 0.0;
            }
        }
    }

    score -= swamp_percent * 1.5;

    // Too much swamp = unusable.
    if swamp_percent >= 45.0 {
        return 0.0;
    }

    score
}

pub fn nearby_source_keepers(room_name: &RoomName) -> f64 {
    let nearby_rooms = room_name.get_adjacent(1_i32);

    let mut amount = 0.0;

    for room in nearby_rooms {
        let room_type = utils::room_type(&room);

        if room_type == RoomType::SourceKeeper {
            amount += 1.0;
        }
    }

    amount
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

    (highest_xy, highest_position >= 7)
}

pub fn scan_remote_accessibility(room_name: &RoomName) -> u32 {
    let potential_remotes = room_name.get_adjacent(2 as i32);

    let mut accessible = 0;
    for remote in potential_remotes {
        let path = game::map::find_route(*room_name, remote, Some(FindRouteOptions::default()));

        if path.is_ok() {
            accessible += path.unwrap().len() as u32;
        }
    }

    accessible
}

pub fn score_remotes(room_name: &RoomName, memory: &ScreepsMemory) -> (f64, u32) {
    let potential_remotes = room_name.get_adjacent(2 as i32);
    let mut score = 0.0;
    let mut source_count = 0;

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
                    if scouting_data.room_type == RoomType::Normal {
                        source_count += 1;
                    }

                    let opts = SearchOptions::default().max_ops(2000000);
                    let target = MoveTarget {
                        pos: room_pos,
                        range: 24,
                    }
                    .pathfind(source.pos.as_position(&remote), Some(opts));

                    if !target.incomplete() {
                        paths.push(target.path().len());
                    } else {
                        paths.push((room_pos.get_range_to(source.pos.as_position(&remote)) * 3).try_into().unwrap_or(0));
                    }
                }
            }
        }

        if game::map::get_room_status(remote).is_some() && game::map::get_room_status(remote).unwrap().status() == RoomStatus::Normal {
            let swampiness = utils::calculate_swamp_percentage(&remote);
            paths.push(swampiness as usize);
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

    (score, source_count)
}
