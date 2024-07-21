use screeps::{Creep, Part, SharedCreepProperties};

pub fn get_healer(creeps: &Vec<Creep>) -> Option<Creep> {
    let mut highest_heal_count = u32::MIN;
    let mut healer = 0;

    for creep in creeps {
        let heal_count = creep.body().iter().filter(|part| part.part() == Part::Heal).count() as u32;

        if heal_count > highest_heal_count {
            highest_heal_count = heal_count;
            healer = creeps.iter().position(|c| c.name() == creep.name()).unwrap();
        }
    }

    creeps.get(healer).cloned()
}

pub fn get_attacker(creeps: &Vec<Creep>) -> Option<Creep> {
    let mut highest_attack_count = u32::MIN;
    let mut attacker = 0;

    for creep in creeps {
        let attack_count = creep.body().iter().filter(|part| part.part() == Part::Attack || part.part() == Part::RangedAttack || part.part() == Part::Work).count() as u32;

        if attack_count > highest_attack_count {
            highest_attack_count = attack_count;
            attacker = creeps.iter().position(|c| c.name() == creep.name()).unwrap();
        }
    }

    creeps.get(attacker).cloned()
}