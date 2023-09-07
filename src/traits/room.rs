use std::collections::HashMap;

use log::info;
use regex::Regex;
use screeps::{CostMatrix, Terrain, Creep, find, SharedCreepProperties, MaybeHasTypedId, StructureProperties, HasTypedId};

use crate::{ALLIES, cache::ScreepsCache};

pub trait RoomExtensions {
    fn name_str(&self) -> String;

    fn initialise_cache(&self, cache: &mut ScreepsCache);
    fn get_enemy_creeps(&self) -> Vec<Creep>;

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

    fn initialise_cache(&self, cache: &mut ScreepsCache) {
        let enemy_creeps = self.find(find::HOSTILE_CREEPS, None);
        let structures = self.find(find::STRUCTURES, None);
        let construction_sites = self.find(find::CONSTRUCTION_SITES, None);
        let energy = self.find(find::DROPPED_RESOURCES, None);
        if cache.room_specific.get(&self.name_str()).is_none() {
            cache.room_specific.insert(
                self.name_str(),
                crate::cache::RoomSpecific {
                    enemy_creeps: Vec::new(),
                    towers: Vec::new(),
                    structures: HashMap::new(),
                    csites: Vec::new(),
                    cost_matrix: None,
                    energy: Vec::new(),
                },
            );
        }
        for creep in enemy_creeps {
            cache.room_specific.get_mut(&self.name_str()).unwrap().enemy_creeps.push(creep.try_id().unwrap());
        }
        for structure in structures {
            match cache.room_specific.get_mut(&self.name_str()).unwrap().structures.entry(structure.structure_type()) {
                std::collections::hash_map::Entry::Occupied(v) => v.into_mut().push(structure.as_structure().try_id().unwrap()),
                std::collections::hash_map::Entry::Vacant(_) => {
                    cache.room_specific.get_mut(&self.name_str()).unwrap().structures.insert(structure.structure_type(), vec![structure.as_structure().try_id().unwrap()]);
                }
            }
        }
        for csite in construction_sites {
            cache.room_specific.get_mut(&self.name_str()).unwrap().csites.push(csite.try_id().unwrap());
        }
        for energy in energy {
            cache.room_specific.get_mut(&self.name_str()).unwrap().energy.push(energy.id());
        }
    }

    fn get_enemy_creeps(&self) -> Vec<Creep> {
        self.find(screeps::constants::find::HOSTILE_CREEPS, None).into_iter().filter(|c| !ALLIES.contains(&c.owner().username().to_string().as_str())).collect()
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
