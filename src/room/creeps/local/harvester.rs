use log::info;
use screeps::{Source, HasPosition, Creep, ResourceType, Part, SharedCreepProperties, game};

use crate::{memory::ScreepsMemory, traits::creep::CreepExtensions, cache::ScreepsCache};

pub fn run_creep(creep: &Creep, memory: &mut ScreepsMemory, source: Source, cache: &mut ScreepsCache) {
    let start_time = game::cpu::get_used();
    let owning_room = memory.get_creep(&creep.name()).o_r.clone();
    if creep.better_is_near(source.pos()) <= 1 {

        if creep.store().get_free_capacity(Some(ResourceType::Energy)) > creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32 {

            let _ = creep.drop(ResourceType::Energy, Some(creep.store().get_used_capacity(Some(ResourceType::Energy))));
            info!("Drop time: {}", game::cpu::get_used() - start_time);

        } else {

        creep.harvest(&source).unwrap_or(());
        let energy_harvested = std::cmp::min(creep.body().iter().filter(|b| b.part() == Part::Work).count() as u32 * 2, source.energy()) as u64;

        memory.stats.get_room(&owning_room).energy_harvested += energy_harvested;
        memory.stats.get_room(&owning_room).energy_harvested_total += energy_harvested;
        memory.stats.energy_harvested += energy_harvested;
        info!("Harvest time: {}", game::cpu::get_used() - start_time);
    }
    } else {

        creep.better_move_to(memory.get_creep(&creep.name()), cache, source.pos(), 1);
        info!("Move time (Harvester): {}", game::cpu::get_used() - start_time);
    }
}