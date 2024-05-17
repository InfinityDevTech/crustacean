use screeps::{game, OwnedStructureProperties};

pub fn distance_formula(source: (u32, u32), target: (u32, u32)) -> u32 {
    let x = source.0 as i32 - target.0 as i32;
    let y = source.1 as i32 - target.1 as i32;
    ((x.pow(2) + y.pow(2)) as f64).sqrt() as u32
}

pub fn just_reset() -> bool {
    if game::time() == 0 { return true; }

    if game::creeps().entries().count() >= 1 { return false; }
    if game::rooms().entries().count() > 1 { return false; }

    let room = game::rooms().values().next().unwrap();

    if room.controller().is_none() || !room.controller().unwrap().my() || room.controller().unwrap().level() != 1 || room.controller().unwrap().progress().unwrap() > 0 || room.controller().unwrap().safe_mode().unwrap() > 0 {
        return false;
    }

    if game::spawns().entries().count() != 1 { return false; }

    true
}