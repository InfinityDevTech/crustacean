use std::collections::HashMap;

use screeps::{find, game, HasHits, HasPosition, LocalCostMatrix, Room, RoomXY, Terrain};

use crate::{constants, memory::ScreepsMemory, room::cache::CachedRoom, traits::{position::PositionExtensions, room::find_pos_in_rect}, utils};

pub fn should_safemode(room: &Room, room_cache: &mut CachedRoom, memory: &mut ScreepsMemory) -> bool {
    let mut only_invader = true;
    for attacker in &room_cache.creeps.enemy_creeps_with_attack {
        if attacker.owner().username().to_lowercase() != constants::INVADER_USERNAME.to_lowercase() {
            only_invader = false;
            break;
        }
    }

    if only_invader {
        return false;
    }

    // Jank but works ig, if spawn is below max health
    // and there are attackers, just safemode.
    for spawn in room_cache.structures.spawns.values() {
        if spawn.hits() < spawn.hits_max() && !room_cache.creeps.enemy_creeps_with_attack.is_empty() {
            return true;
        }
    }

    if room_cache.rcl >= 4 && game::time() % 10 == 0 && !room_cache.creeps.enemy_creeps_with_attack.is_empty() {
        let exits = room.find(find::EXIT, None);
        let mut seeds = Vec::new();

        for exit_tile in exits {
            seeds.push((exit_tile.x(), exit_tile.y()))
        }

        let mut flood_cm = LocalCostMatrix::new();
        let terrain = room.get_terrain();
        let mut visited_cms = LocalCostMatrix::new();

        let mut depth = 0;
        let mut this_gen = seeds.clone();
        let mut next_gen = Vec::new();
        for (x, y) in &seeds {
            let xy = utils::new_xy(*x, *y);
            room.visual().circle(*x as f32, *y as f32, None);
            visited_cms.set(xy, 1);
        }

        let mut ramparts_at_pos = HashMap::new();
        for ramparts in &room_cache.structures.ramparts {
            ramparts_at_pos.insert(ramparts.pos().xy(), ramparts.clone());
        }

        while !this_gen.is_empty() {
            next_gen.clear();

            for (x, y) in &this_gen.clone() {
                if *x > 49 || *y > 49 {
                    continue;
                }

                let xy = utils::new_xy(*x, *y);

                if depth != 0 {
                    if terrain.get(*x, *y) == Terrain::Wall || ramparts_at_pos.contains_key(&xy) {
                        continue;
                    }

                    flood_cm.set(xy, depth);
                }

                let rect = (x - 1, y - 1, x + 1, y + 1);
                let adjacent_psoitions = find_pos_in_rect(rect);

                for (x2, y2) in adjacent_psoitions {
                    let xy = utils::new_xy(x2, y2);

                    if visited_cms.get(xy) == 1 {
                        continue;
                    }

                    visited_cms.set(xy, 1);

                    next_gen.push((x2, y2));
                }
            }
            this_gen = next_gen.clone();
            depth += 1;
        }

        for (spawn_id, spawn) in &room_cache.structures.spawns {
            let pos = spawn.pos().get_accessible_positions_around(1);

            for pos in pos {
                let xy = utils::new_xy(pos.x().u8(), pos.y().u8());

                // 3 is arbitrary, just has to be above 0.
                if flood_cm.get(xy) > 3 {

                    let mut is_near = false;
                    for hostile in &room_cache.creeps.enemy_creeps_with_attack {
                        if let Some(spawn_center) = room_cache.spawn_center {
                            if hostile.pos().xy().get_range_to(spawn_center) <= 10 {
                                is_near = true;
                                break;
                            }
                        }
                    }

                    if is_near {
                        return true;
                    } else {
                        break;
                    }
                }
            }
        }
    }


    false
}