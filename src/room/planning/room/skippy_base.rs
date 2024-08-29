// Mappings:
// 'c' = container
// 'r' = road
// 'e' = extension
// 'p' = spawn
// 'l' = link
// 'q' = terminal
// 'n' = nuker
// 'f' = factory
// 's' = storage
// 'z' = power spawn
// 'o' = observer
// 'x' = lab
// 't' = tower

use std::collections::HashMap;

use log::info;
use screeps::{find, game, CircleStyle, HasId, HasPosition, Room, StructureType, TextStyle};

use crate::{
    memory::{RoomMemory, ScreepsMemory, SkippyMem},
    room::visuals,
};

use super::structure_visuals::{self, RoomVisualExt};

pub fn stamp_list() -> Vec<(u8, Vec<Vec<char>>)> {
    vec![
        (4, ff_stamp()),
        (3, store_stamp()),
        (4, lab_stamp()),
        (4, large_ext_stamp()),
        (3, ext_stamp()),
        (3, ext_stamp()),
        (3, ext_stamp()),
        (3, ext_stamp()),
        (3, ext_stamp()),
        (3, ext_stamp()),
    ]
}

pub fn large_ext_stamp() -> Vec<Vec<char>> {
    vec![
        vec![' ', ' ', ' ', 'r', ' ', ' ', ' '],
        vec![' ', ' ', 'r', 'e', 'r', ' ', ' '],
        vec![' ', 'r', 'e', 'e', 'e', 'r', ' '],
        vec!['r', 'e', 'e', ' ', 'e', 'e', 'r'],
        vec![' ', 'r', 'e', 'e', 'e', 'r', ' '],
        vec![' ', ' ', 'r', 'e', 'r', ' ', ' '],
        vec![' ', ' ', ' ', 'r', ' ', ' ', ' '],
    ]
}

pub fn ff_stamp() -> Vec<Vec<char>> {
    vec![
        vec!['r', 'r', 'r', 'r', 'r', 'r', 'r'],
        vec!['r', 'e', 'p', 'e', 'p', 'e', 'r'],
        vec!['r', 'c', ' ', 'l', ' ', 'c', 'r'],
        vec!['r', 'e', 'e', 'p', 'e', 'e', 'r'],
        vec!['r', 'r', 'r', 'r', 'r', 'r', 'r'],
    ]
}

pub fn store_stamp() -> Vec<Vec<char>> {
    vec![
        vec!['r', 'r', 'r', 'r', 'r'],
        vec!['r', 'l', 'r', 'q', 'r'],
        vec!['r', 'n', ' ', 'f', 'r'],
        vec!['r', 's', 'z', 'o', 'r'],
        vec!['r', 'r', 'r', 'r', 'r'],
    ]
}

pub fn lab_stamp() -> Vec<Vec<char>> {
    vec![
        vec![' ', ' ', ' ', 'r', ' ', ' ', ' '],
        vec![' ', ' ', 'r', 'x', 'r', ' ', ' '],
        vec![' ', 'r', 'x', 'x', 'r', 'r', ' '],
        vec!['r', 'r', 'x', 'x', 'x', 'x', 'r'],
        vec![' ', 'r', 'x', 'x', 'x', 'r', ' '],
        vec![' ', ' ', 'r', 'r', 'r', ' ', ' '],
        vec![' ', ' ', ' ', 'r', ' ', ' ', ' '],
    ]
}

pub fn ext_stamp() -> Vec<Vec<char>> {
    vec![
        vec![' ', ' ', 'r', ' ', ' '],
        vec![' ', 'r', 'e', 'r', ' '],
        vec!['r', 'e', 't', 'e', 'r'],
        vec![' ', 'r', 'e', 'r', ' '],
        vec![' ', ' ', 'r', ' ', ' '],
    ]
}

pub fn setup_plan_room(room: &Room, memory: &mut SkippyMem) -> bool {
    let mut plan_arr = Vec::new();

    let terrain = room.get_terrain().get_raw_buffer().to_vec();
    for i in 0..2500 {
        if terrain[i] & 1 == 1 {
            plan_arr.push('W');
        } else {
            plan_arr.push(' ');
        }
    }

    let offsets: Vec<i32> = vec![-51, -50, -49, -1, 1, 49, 50, 51];
    for i in 0..50 {
        for j in [i, 2450 + i, i * 50, i * 50 + 49] {
            if terrain[j] & 1 != 1 {
                plan_arr[j] = 'E';
                for offset in &offsets {
                    if j as i32 + offset > plan_arr.len() as i32 {
                        continue;
                    }

                    if plan_arr[(j as i32 + offset) as usize] == ' ' {
                        plan_arr[(j as i32 + offset) as usize] = 'E'
                    }
                }
            }
        }
    }

    memory.map = plan_arr;
    true
}

pub fn run_planner(room: &Room, memory: &mut RoomMemory) {
    if memory.skippy_planner.is_none() {
        memory.skippy_planner = Some(SkippyMem {
            map: Vec::new(),
            step: 0,
            source_fills: HashMap::new(),
            controller_fill: HashMap::new(),
            orth_wall_fill: HashMap::new(),
            core: 0,
            stamp_index: None,
            source_labs: [0; 2],
            planned: false,
        });
    }

    if game::flags().get("resetPlanner".to_string()).is_some() {
        memory.skippy_planner = None;
        return;
    }

    let mut plan_mem = &mut memory.skippy_planner.as_mut().unwrap();

    if plan_mem.planned {
        if game::flags().get("visOrth".to_string()).is_some() {
            let fill = &plan_mem.orth_wall_fill;
            for i in 0..2500 {
                if fill[&i] < 255 {
                    let red = (255).min(fill[&i] * 5).to_string();
                    let non_red = (255 - (255).min(fill[&i] * 5)).to_string();
                    room.visual().text(
                        (i % 50) as f32,
                        (i as f32 / 50.0).round(),
                        fill[&i].to_string(),
                        Some(
                            TextStyle::default()
                                .custom_font("0.6 serif")
                                .color(&format!("rgb({}, {}, {})", red, non_red, non_red)),
                        ),
                    )
                }
            }
        } else {
            for i in 0..2500 {
                let mut vis = RoomVisualExt::new(room.name());
                let x = i % 50;
                let y = (i as f32 / 50.0).floor() as i32;

                let stringified = plan_mem.map[i];

                if stringified == 'W' {
                    room.visual()
                        .text(x as f32, y as f32, "W".to_string(), None);
                }

                let s_type = match stringified {
                    'c' => StructureType::Container,
                    'r' => StructureType::Road,
                    'l' => StructureType::Link,
                    'p' => StructureType::Spawn,
                    'e' => StructureType::Extension,
                    'x' => StructureType::Lab,
                    't' => StructureType::Tower,
                    'q' => StructureType::Terminal,
                    'n' => StructureType::Nuker,
                    'f' => StructureType::Factory,
                    's' => StructureType::Storage,
                    'z' => StructureType::PowerSpawn,
                    'o' => StructureType::Observer,
                    _ => StructureType::Controller,
                };

                vis.structure(x as f32, y as f32, s_type, 0.5);
            }
        }
        return;
    }

    let ret = match plan_mem.step {
        0 => setup_plan_room(room, &mut plan_mem),
        1 => source_fills(room, &mut plan_mem),
        2 => controller_fill(room, &mut plan_mem),
        3 => orth_wall_fill(room, &mut plan_mem),
        4 => place_stamp(room, &mut plan_mem),
        5 => finialize(room, &mut plan_mem),
        _ => {
            plan_mem.step = 0;
            false
        }
    };

    if ret {
        plan_mem.step += 1;
    }
}

pub fn source_fills(room: &Room, memory: &mut SkippyMem) -> bool {
    let sources = room.find(find::SOURCES, None);

    for source in sources {
        if let std::collections::hash_map::Entry::Vacant(e) = memory.source_fills.entry(source.id())
        {
            let mut fill: HashMap<i32, i32> = HashMap::new();

            let terrain = room.get_terrain().get_raw_buffer().to_vec();
            for (index, t) in terrain.iter().enumerate().take(2500) {
                if *t & 1 == 1 {
                    fill.insert(index as i32, 255);
                } else {
                    fill.insert(index as i32, 0);
                }
            }

            for i in 0..50 {
                for j in [i, 2450 + i, i * 50, i * 50 + 49] {
                    if fill[&j] == 0 {
                        fill.insert(j as i32, -1);
                    }
                }
            }

            let mut steps: HashMap<i32, i32> = HashMap::new();
            let source_index = source.pos().x().u8() as i32 * 50 + source.pos().y().u8() as i32;
            let offset = [-51, -50, -49, -1, 1, 49, 50, 51];
            for off in offset {
                fill.insert(source_index + off, 1);
            }

            for i in 0..100 {
                let mut next_steps = HashMap::new();
                for j in steps.keys() {
                    if fill[j] == 0 {
                        fill.insert(*j, i + 1);
                        for off in offset {
                            let k = j + off;
                            next_steps.insert(k, 1);
                        }
                    } else if fill[j] == -1 {
                        fill.insert(*j, i + 1);

                        let x = j % 50;
                        let y = (*j as f32 / 50.0).floor() as i32;

                        if x == 0 {
                            for off in [-49, 1, 51] {
                                let k = j + off;
                                next_steps.insert(k, 1);
                            }
                        } else if x == 49 {
                            for off in [-51, -1, 49] {
                                let k = j + off;
                                next_steps.insert(k, 1);
                            }
                        } else if y == 0 {
                            for off in [49, 50, 51] {
                                let k = j + off;
                                next_steps.insert(k, 1);
                            }
                        } else if y == 49 {
                            for off in [-51, -50, -49] {
                                let k = j + off;
                                next_steps.insert(k, 1);
                            }
                        }
                    }
                }

                steps = next_steps
            }

            if true {
                for i in 0..2500 {
                    if fill[&i] < 255 {
                        let red = (255).min(fill[&i] * 5).to_string();
                        let non_red = (255 - (255).min(fill[&i] * 5)).to_string();
                        room.visual().text(
                            (i % 50) as f32,
                            (i as f32 / 50.0).round(),
                            fill[&i].to_string(),
                            Some(
                                TextStyle::default()
                                    .custom_font("0.6 serif")
                                    .color(&format!("rgb({}, {}, {})", red, non_red, non_red)),
                            ),
                        )
                    }
                }
            }

            e.insert(fill);
            return false;
        }
    }
    true
}

pub fn controller_fill(room: &Room, memory: &mut SkippyMem) -> bool {
    let mut fill: HashMap<i32, i32> = HashMap::new();

    let terrain = room.get_terrain().get_raw_buffer().to_vec();
    for (index, t) in terrain.iter().enumerate().take(2500) {
        if *t & 1 == 1 {
            fill.insert(index as i32, 255);
        } else {
            fill.insert(index as i32, 0);
        }
    }

    for i in 0..50 {
        for j in [i, 2450 + i, i * 50, i * 50 + 49] {
            if fill[&j] == 0 {
                fill.insert(j, -1);
            }
        }
    }

    let mut steps: HashMap<i32, i32> = HashMap::new();
    let controller_index =
        room.controller().unwrap().pos().y().u8() * 50 + room.controller().unwrap().pos().x().u8();
    let offset = [-51, -50, -49, -1, 1, 49, 50, 51];

    for off in offset {
        steps.insert(controller_index as i32 + off, 1);
    }
    for i in 0..100 {
        let mut next_steps: HashMap<i32, i32> = HashMap::new();
        for j in steps.keys() {
            if fill[j] == 0 {
                fill.insert(*j, i + 1);
                for off in offset {
                    let k = j + off;
                    next_steps.insert(k, 1);
                }
            } else if fill[j] == -1 {
                fill.insert(*j, i + 1);

                let x = j % 50;
                let y = (*j as f32 / 50.0).floor() as i32;

                if x == 0 {
                    for off in [-49, 1, 51] {
                        let k = j + off;
                        next_steps.insert(k, 1);
                    }
                } else if x == 49 {
                    for off in [-51, -1, 49] {
                        let k = j + off;
                        next_steps.insert(k, 1);
                    }
                } else if y == 0 {
                    for off in [49, 50, 51] {
                        let k = j + off;
                        next_steps.insert(k, 1);
                    }
                } else if y == 49 {
                    for off in [-51, -50, -49] {
                        let k = j + off;
                        next_steps.insert(k, 1);
                    }
                }
            }
        }

        steps = next_steps;
    }

    if true {
        for i in 0..2500 {
            if fill[&i] < 255 {
                let red = (255).min(fill[&i] * 5).to_string();
                let non_red = (255 - (255).min(fill[&i] * 5)).to_string();
                room.visual().text(
                    (i % 50) as f32,
                    (i as f32 / 50.0).round(),
                    fill[&i].to_string(),
                    Some(
                        TextStyle::default()
                            .custom_font("0.6 serif")
                            .color(&format!("rgb({}, {}, {})", red, non_red, non_red)),
                    ),
                )
            }
        }
    }

    memory.controller_fill = fill;
    true
}

pub fn orth_wall_fill(room: &Room, memory: &mut SkippyMem) -> bool {
    let mut fill: HashMap<i32, i32> = HashMap::new();

    let terrain = room.get_terrain().get_raw_buffer().to_vec();
    for (index, t) in terrain.iter().enumerate().take(2500) {
        if [' ', 'r'].contains(&memory.map[index]) {
            fill.insert(index as i32, 0);
        } else {
            fill.insert(index as i32, 255);
        }
    }

    let mut steps: HashMap<i32, i32> = HashMap::new();
    let offset = [-50, -1, 1, 50];
    for x in 0..50 {
        for y in 0..50 {
            let index = y * 50 + x;
            if fill[&index] == 255 {
                if x == 0 {
                    for off in [-50, 1, 50] {
                        steps.insert(index + off, 1);
                    }
                } else if x == 49 {
                    for off in [-50, -1, 50] {
                        steps.insert(index + off, 1);
                    }
                } else if y == 0 {
                    for off in [-1, 50, 1] {
                        steps.insert(index + off, 1);
                    }
                } else if y == 49 {
                    for off in [-1, -50, 1] {
                        steps.insert(index + off, 1);
                    }
                } else {
                    for off in offset {
                        steps.insert(index + off, 1);
                    }
                }
            }
        }
    }

    for i in 0..100 {
        let mut next_steps: HashMap<i32, i32> = HashMap::new();
        for j in steps.keys() {
            if !fill.contains_key(j) {
                info!("{} is not contained...", j);

                continue;
            }

            if fill[j] == 0 {
                fill.insert(*j, i + 1);
                for off in offset {
                    let k = j + off;
                    next_steps.insert(k, 1);
                }
            } else if fill[j] == -1 {
                fill.insert(*j, i + 1);

                let x = j % 50;
                let y = (*j as f32 / 50.0).round() as i32;
                if x == 0 {
                    for off in [-50, 1, 50] {
                        let k = j + off;
                        next_steps.insert(k, 1);
                    }
                } else if x == 49 {
                    for off in [-50, -1, 50] {
                        let k = j + off;
                        next_steps.insert(k, 1);
                    }
                } else if y == 0 {
                    for off in [-1, 50, 1] {
                        let k = j + off;
                        next_steps.insert(k, 1);
                    }
                } else if y == 49 {
                    for off in [-1, -50, 1] {
                        let k = j + off;
                        next_steps.insert(k, 1);
                    }
                }
            }
        }
        steps = next_steps;
    }

    if true {
        for i in 0..2500 {
            if fill[&i] < 255 {
                let red = (255).min(fill[&i] * 5).to_string();
                let non_red = (255 - (255).min(fill[&i] * 5)).to_string();
                room.visual().text(
                    (i % 50) as f32,
                    (i as f32 / 50.0).round(),
                    fill[&i].to_string(),
                    Some(
                        TextStyle::default()
                            .custom_font("0.6 serif")
                            .color(&format!("rgb({}, {}, {})", red, non_red, non_red)),
                    ),
                )
            }
        }
    }

    memory.orth_wall_fill = fill;

    true
}

pub fn place_stamp(room: &Room, memory: &mut SkippyMem) -> bool {
    if memory.stamp_index.is_none() {
        memory.stamp_index = Some(0);
    }

    let current_stamp_size = stamp_list()[memory.stamp_index.unwrap() as usize].0;
    let current_stamp_layout = &stamp_list()[memory.stamp_index.unwrap() as usize].1;
    let mut best_spot = 0;
    let mut lowest_score = 9001;

    for i in 0..2500 {
        if memory.orth_wall_fill[&i] >= current_stamp_size.into() && memory.orth_wall_fill[&i] < 255
        {
            let mut score = memory.controller_fill[&i] * 3;
            for s_id in memory.source_fills.keys() {
                score += memory.source_fills[s_id][&i] * 2
            }
            if score < lowest_score {
                lowest_score = score;
                best_spot = i;
            }
        }
    }

    if best_spot > 0 {
        if memory.stamp_index == Some(0) {
            memory.core = best_spot;
        }
        if memory.stamp_index == Some(2) {
            memory.source_labs = [best_spot - 50, best_spot + 1];
        }

        let anchor_x = best_spot % 50 - (current_stamp_size - 1) as i32;
        let anchor_y = (best_spot as f32 / 50.0).floor() as i32 - (current_stamp_size - 1) as i32;

        for i in 0..current_stamp_layout.len() {
            for j in 0..current_stamp_layout[i].len() {
                let x = anchor_x + j as i32;
                let y = anchor_y + i as i32;

                if current_stamp_layout[i][j] != ' ' {
                    memory.map[(y * 50 + x) as usize] = current_stamp_layout[i][j];
                }
            }
        }
    } else {
        info!("Uhmm, yeah, not working... Best spot is not > 0. Kill me");
    }

    if true {
        for i in 0..2500 {
            if memory.orth_wall_fill[&i] < 255 {
                let red = (255).min(memory.orth_wall_fill[&i] * 5).to_string();
                let non_red = (255 - (255).min(memory.orth_wall_fill[&i] * 5)).to_string();
                room.visual().text(
                    (i % 50) as f32,
                    (i as f32 / 50.0).round(),
                    memory.orth_wall_fill[&i].to_string(),
                    Some(
                        TextStyle::default()
                            .custom_font("0.6 serif")
                            .color(&format!("rgb({}, {}, {})", red, non_red, non_red)),
                    ),
                )
            }
        }
        room.visual().circle(
            best_spot as f32 % 50.0,
            (best_spot as f32 / 50.0).floor(),
            Some(CircleStyle::default().radius(0.5).fill("#ff0000")),
        )
    }

    memory.stamp_index = Some(memory.stamp_index.unwrap() + 1);
    if stamp_list().len() > memory.stamp_index.unwrap() as usize {
        memory.step -= 1;
        return false;
    } else {
        info!("len {}", stamp_list().len());
        info!("Index {}", memory.stamp_index.unwrap());
    }

    true
}

pub fn finialize(room: &Room, memory: &mut SkippyMem) -> bool {
    memory.planned = true;

    return false;
}
