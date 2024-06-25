// This is done because since its in config, it CAN be changed to "" and then it will be empty
#![allow(clippy::comparison_to_empty)]

use log::info;
use regex::Regex;
use screeps::{CostMatrix, OwnedStructureProperties, RoomName, Terrain};
use serde::{Deserialize, Serialize};

use crate::{config, room::cache::tick_cache::CachedRoom};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RoomType {
    Normal,
    Highway,
    Intersection,
    SourceKeeper,
    Center,
    Unknown,
}

pub trait RoomExtensions {
    fn name_str(&self) -> String;
    fn split_name(&self) -> (String, u32, String, u32);
    fn my(&self) -> bool;

    fn get_target_for_miner(&self, cache: &mut CachedRoom) -> Option<u8>;

    fn is_my_sign(&self) -> bool;

    fn get_adjacent(&self, radius: u32) -> Vec<RoomName>;

    fn room_type(&self) -> RoomType;
    fn is_highway(&self) -> bool;
    fn is_intersection(&self) -> bool;
    fn is_source_keeper(&self) -> bool;

    fn flood_fill(&self, seeds: Vec<(u8, u8)>) -> CostMatrix;
}

//#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomExtensions for screeps::Room {
    fn name_str(&self) -> String {
        self.name().to_string()
    }
    fn split_name(&self) -> (String, u32, String, u32) {
        let room_regex = Regex::new("^([WE]{1})([0-9]{1,2})([NS]{1})([0-9]{1,2})$").unwrap();
        let room_name = self.name_str();

        let captures = room_regex.captures(&room_name).unwrap();

        (
            captures[1].to_string(),
            captures[2].to_string().parse::<u32>().unwrap(),
            captures[3].to_string(),
            captures[4].to_string().parse::<u32>().unwrap(),
        )
    }
    fn my(&self) -> bool {
        self.controller()
            .map_or(false, |controller| controller.my())
    }

    fn get_adjacent(&self, radius: u32) -> Vec<RoomName> {
        let split_name = self.split_name();

        let start_x = split_name.1;
        let start_y = split_name.3;

        let mut adjacent_rooms = vec![];

        for x in start_x - radius..=start_x + radius {
            for y in start_y - radius..=start_y + radius {
                if x == start_x && y == start_y {
                    continue;
                }

                let room_name = format!("{}{}{}{}", split_name.0, x, split_name.2, y);
                adjacent_rooms.push(RoomName::new(&room_name));
            }
        }

        let mut adjacent_checked = Vec::new();
        for room in adjacent_rooms.into_iter().flatten() {
            adjacent_checked.push(room);
        }

        adjacent_checked
    }

    fn get_target_for_miner(&self, room_cache: &mut CachedRoom) -> Option<u8> {
        let sources = &room_cache.resources.sources;

        for (i, source) in sources.iter().enumerate() {
            if source.calculate_work_parts() < source.parts_needed() && source.creeps.len() < source.calculate_mining_spots(self).into() {
                return Some(i as u8);
            }
        }

        None
    }

    fn is_my_sign(&self) -> bool {
        if self.controller().is_none() {
            return true;
        }

        let sign = self.controller().unwrap().sign();
        if sign.is_none() {
            return false;
        }

        let sign_text = self.controller().unwrap().sign().unwrap().text();

        if config::ALLIANCE_TAG != "" && !sign_text.contains(config::ALLIANCE_TAG) {
            return false;
        }

        let tag_without_alliance_marker = if config::ALLIANCE_TAG != "" && sign_text.contains(config::ALLIANCE_TAG) {
            let alliance_marker = format!("{} ", config::ALLIANCE_TAG);
            sign_text.replace(&alliance_marker, "")
        } else {
            sign_text.to_string()
        };

        config::ROOM_SIGNS.contains(&tag_without_alliance_marker.as_str())
    }

    fn room_type(&self) -> RoomType {
            let room_x = self.name().x_coord();
            let room_y = self.name().y_coord();
        
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

    fn is_highway(&self) -> bool {
        let split_name = self.split_name();
        let east_west_distance = split_name.1;
        let north_south_distance = split_name.3;

        if east_west_distance % 10 == 0 || north_south_distance % 10 == 0 {
            return true;
        }
        false
    }
    fn is_intersection(&self) -> bool {
        let split_name = self.split_name();
        let east_west_distance = split_name.1;
        let north_south_distance = split_name.3;

        if east_west_distance % 10 == 0 && north_south_distance % 10 == 0 {
            return true;
        }
        false
    }
    fn is_source_keeper(&self) -> bool {
        let split_name = self.split_name();
        let east_west_distance = split_name.1;
        let north_south_distance = split_name.3;

        if east_west_distance % 10 == 5
            && north_south_distance % 10 == 5
            && !north_south_distance % 10 == 0
            && !east_west_distance % 10 == 0
        {
            return true;
        }
        false
    }

    fn flood_fill(&self, seeds: Vec<(u8, u8)>) -> CostMatrix {
        let flood_cm = CostMatrix::new();
        let terrain = self.get_terrain();
        let visited_cms = CostMatrix::new();

        info!("Flood fill started on room {}", self.name_str());

        let mut depth = 0;
        let mut this_gen = seeds.clone();
        let mut next_gen = Vec::new();
        for (x, y) in &seeds {
            visited_cms.set(*x, *y, 1);
            info!("Seed: {}, {}", x, y);
        }

        while !this_gen.is_empty() {
            next_gen.clear();

            for (x, y) in &this_gen.clone() {
                if depth != 0 {
                    if terrain.get(*x, *y) == Terrain::Wall {
                        continue;
                    }

                    flood_cm.set(*x, *y, depth);
                }

                let rect = (x - 1, y - 1, x + 1, y + 1);
                let adjacent_psoitions = find_pos_in_rect(rect);

                for (x2, y2) in adjacent_psoitions {
                    if visited_cms.get(x2, y2) == 1 {
                        continue;
                    }

                    visited_cms.set(x2, y2, 1);

                    next_gen.push((x2, y2));
                }
            }
            this_gen.clear();
            this_gen.clone_from(&next_gen);
            depth += 1;
        }

        flood_cm
    }
}

pub fn find_pos_in_rect(rect: (u8, u8, u8, u8)) -> Vec<(u8, u8)> {
    let mut positions = Vec::new();

    for x in rect.0..=rect.2 {
        for y in rect.1..=rect.3 {
            if x >= 50 || y >= 50 {
                continue;
            }
            positions.push((x, y));
        }
    }

    positions
}
