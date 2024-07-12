#![allow(dead_code)]

use log::info;
use screeps::{Source, StructureSpawn, HasPosition, pathfinder::{SearchOptions, MultiRoomCostResult, self}, RoomName, LocalCostMatrix, game, StructureType, find, StructureProperties, look};

use crate::traits::intents_tracking::RoomExtensionsTracking;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn source_to_spawn(source: &Source, spawn: &StructureSpawn) {
    let opts = SearchOptions::new(road_callback).max_ops(100000000).plain_cost(2).swamp_cost(5).max_rooms(1);
    let path = pathfinder::search(spawn.pos(), source.pos(), 1, Some(opts));
    if !path.incomplete() {
        info!("Road complete");
        for pos in path.path() {
            let room = game::rooms().get(pos.room_name()).unwrap();
            if room.look_for_at_xy(look::CONSTRUCTION_SITES, pos.x().u8(), pos.y().u8()).is_empty() {
                match room.ITcreate_construction_site(pos.x().u8(), pos.y().u8(), StructureType::Road, None) {
                    Ok(_) => {},
                    Err(e) => {
                        println!("Error creating construction site: {:?}", e);
                    }
                };
            }
        }
    } else {
        info!("Road incomplete?");
    }
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn road_callback(room_name: RoomName) -> MultiRoomCostResult {
    let mut matrix = LocalCostMatrix::new();
    if let Some(room) = game::rooms().get(room_name) {
        for road in room.find(find::STRUCTURES, None).into_iter().filter(|s| s.structure_type() == StructureType::Road) {
            matrix.set(road.pos().xy(), 1);
        }
        let csites = room.find(find::CONSTRUCTION_SITES, None);
        for site in csites {
            match site.structure_type() {
                StructureType::Road => matrix.set(site.pos().xy(), 1),
                StructureType::Rampart => matrix.set(site.pos().xy(), 1),
                StructureType::Container => matrix.set(site.pos().xy(), 1),
                _ => todo!(),
            }
        }
        for creep in room.find(find::CREEPS, None) {
            matrix.set(creep.pos().xy(), 1);
        }
    }

    MultiRoomCostResult::CostMatrix(matrix.into())

}