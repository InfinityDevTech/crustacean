use log::info;
use regex::Regex;
use screeps::{CostMatrix, Terrain};

pub trait RoomExtensions {
    fn name_str(&self) -> String;

    fn split_room_name(&self) -> (String, u32, String, u32);

    fn is_highway(&self) -> bool;
    fn is_intersection(&self) -> bool;
    fn is_source_keeper(&self) -> bool;

    fn flood_fill(&self, seeds: Vec<(u8, u8)>) -> CostMatrix;
}

impl RoomExtensions for screeps::Room {
    fn name_str(&self) -> String {
        self.name().to_string()
    }

    fn split_room_name(&self) -> (String, u32, String, u32) {
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

    fn is_highway(&self) -> bool {
        let split_name = self.split_room_name();
        let east_west_distance = split_name.1;
        let north_south_distance = split_name.3;

        if east_west_distance % 10 == 0 || north_south_distance % 10 == 0 {
            return true;
        }
        false
    }

    fn is_intersection(&self) -> bool {
        let split_name = self.split_room_name();
        let east_west_distance = split_name.1;
        let north_south_distance = split_name.3;

        if east_west_distance % 10 == 0 && north_south_distance % 10 == 0 {
            return true;
        }
        false
    }

    fn is_source_keeper(&self) -> bool {
        let split_name = self.split_room_name();
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

        let mut depth = 0;
        let mut this_gen = seeds.clone();
        let mut next_gen = Vec::new();
        for (x, y) in &seeds {
            visited_cms.set(*x, *y, 1);
            info!("Seed: {}, {}", x, y);
        }

        while !this_gen.is_empty() {
            info!("Ruinning");
            next_gen.clear();

            for (x, y) in &this_gen.clone() {
                info!("Checking {}, {}", x, y);
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
            this_gen = next_gen.clone();
            info!("Set");
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
