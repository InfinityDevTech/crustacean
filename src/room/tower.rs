use screeps::{Room, find, StructureType, StructureProperties, StructureObject};

pub fn run_towers(room: &Room) {
    let towers = room.find(find::MY_STRUCTURES, None).into_iter().filter(|s| s.structure_type() == StructureType::Tower).collect::<Vec<StructureObject>>();
    if !towers.is_empty() {
        let enemies = room.find(find::HOSTILE_CREEPS, None);
        if enemies.is_empty() {
            return;
        }
        for tower_obj in towers {
            if let StructureObject::StructureTower(tower) = tower_obj {
                let _ = tower.attack(enemies.first().unwrap());
            }
        }
    }
}