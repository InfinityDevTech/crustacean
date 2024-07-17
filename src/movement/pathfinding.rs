use std::{collections::HashMap, ops::Mul};

use log::info;
use screeps::{
    game, pathfinder::MultiRoomCostResult, CostMatrixGet, Direction, LocalCostMatrix,
    LocalRoomTerrain, Position, RoomCoordinate, RoomName, RoomTerrain, RoomXY, Terrain,
};

use crate::{constants::PATHFINDER_MAX_ROOMS, room};

use super::{
    coord_convert::{
        generate_room_name, null_world_pos, parse_room_name, position_to_world_position,
        world_position, MapPosition, WorldPosition,
    },
    path_heap::{OpenClose, PathHeap},
    movement_utils::num_to_dir,
};

#[derive(Clone)]
pub struct RoomInfo {
    pub terrain: LocalRoomTerrain,
    pub cost_matrix: LocalCostMatrix,
    pub pos: MapPosition,
}

impl RoomInfo {
    pub fn look(&mut self, xx: u8, yy: u8) -> u8 {
        let coord = RoomXY::new(RoomCoordinate::new(xx).unwrap(), RoomCoordinate::new(yy).unwrap());
        if self.cost_matrix.get_xy(coord) != 0 {
            return self.cost_matrix.get_xy(coord);
        }

        return self.terrain.get_xy(coord) as u8;
    }
}

#[derive(Debug)]
pub struct PathFinderSearchResult {
    pub path: Vec<Position>,
    pub cost: u32,
    pub ops: u32,
    pub incomplete: bool,
}

pub struct PathFinderGoal {
    pub pos: WorldPosition,
    pub range: u32,
}

pub struct PathFinder {
    origin: WorldPosition,
    goals: Vec<PathFinderGoal>,
    room_callback: Box<dyn FnMut(RoomName) -> LocalCostMatrix>,
    plain_cost: u8,
    swamp_cost: u8,
    max_rooms: u8,
    max_ops: u32,
    max_cost: u32,
    flee: bool,
    heuristic_weight: f32,

    room_table_size: u32,
    open_close: OpenClose,
    heap: PathHeap,
    parents: Box<[u32; 2500 * PATHFINDER_MAX_ROOMS as usize]>,
    blocked_rooms: HashMap<MapPosition, u32>,
    room_table: Box<[Option<RoomInfo>; PATHFINDER_MAX_ROOMS as usize]>,
    reverse_room_table: Box<[u32; 1 << (size_of::<MapPosition>() * 8)]>,

    look_table: Box<[u8; 4]>,
}

impl PathFinder {
    pub fn setup(
        origin: Position,
        goals: Vec<Position>,
        mut room_callback: Box<dyn FnMut(RoomName) -> LocalCostMatrix>,
        plain_cost: u8,
        swamp_cost: u8,
        max_rooms: u8,
        max_ops: u32,
        max_cost: u32,
        flee: bool,
        heuristic_weight: f32,
    ) -> Self {
        let mut converted_goals = Vec::new();

        for goal in goals {
            converted_goals.push(PathFinderGoal {
                pos: position_to_world_position(&goal),
                range: 0,
            });
        }

        //let mut callback: Box<dyn FnMut(RoomName) -> MultiRoomCostResult> = Box::new(move |room_name| {
        //    (room_callback)(room_name)
        //});

        PathFinder {
            origin: position_to_world_position(&origin),
            goals: converted_goals,
            room_callback,
            plain_cost,
            swamp_cost,
            max_rooms,
            max_ops,
            max_cost,
            flee,
            heuristic_weight,

            room_table_size: 0,
            open_close: OpenClose::new(),
            heap: PathHeap::new(),
            parents: Box::new([0; 2500 * PATHFINDER_MAX_ROOMS as usize]),
            blocked_rooms: HashMap::new(),
            room_table: Box::new([const { None }; PATHFINDER_MAX_ROOMS as usize]),
            reverse_room_table: Box::new([0; 1 << (size_of::<MapPosition>() * 8)]),

            look_table: Box::new([u8::MAX; 4]),
        }
    }

    pub fn search(&mut self) -> PathFinderSearchResult {
        if self.heuristic(&self.origin) == 0 {
            return PathFinderSearchResult {
                path: vec![],
                ops: 0,
                cost: 0,
                incomplete: true,
            };
        }

        let mut ops_remaining = self.max_ops;

        self.look_table[0] = self.plain_cost;
        self.look_table[2] = self.swamp_cost;

        if self.room_index_from_pos(self.origin.map_position()) == 0 {
            return PathFinderSearchResult {
                cost: 0,
                ops: 0,
                incomplete: true,
                path: vec![],
            };
        }

        let min_node = self.room_index_from_pos(self.origin.map_position());
        let origin = self.origin.clone();

        self.astar(min_node, &origin, 0);

        let mut min_node = u32::MAX;
        let mut min_node_h_cost = u32::MAX;
        let mut min_code_g_cost = u32::MAX;

        let igp = attempt_position_reconstruction(&self.origin);
        info!("IGP: {:?}", igp);

        while !self.heap.empty() && ops_remaining > 0 {
            let (index, current) = self.heap.pop();

            self.open_close.close(index);

            let pos = self.pos_from_index(index);
            let h_cost = self.heuristic(&pos);

            let g_cost = current - (h_cost as f32 * self.heuristic_weight) as u32;

            if h_cost == 0 {
                min_node = index;
                min_node_h_cost = 0;
                min_code_g_cost = g_cost;
                break;
            } else if h_cost < min_node_h_cost {
                min_node = index;
                min_node_h_cost = h_cost;
                min_code_g_cost = g_cost;
            }

            if g_cost + h_cost > self.max_cost {
                break;
            }

            self.jump_point_search(index, pos, g_cost);
            ops_remaining -= 1;
        }

        let mut path: Vec<Position> = Vec::new();
        let mut index = min_node;
        let mut pos = self.pos_from_index(index);
        let mut ii = 0;

        info!(
            "Attempting serialization starting at {:?}, min_node: {}, min_node_h_cost: {}, min_code_g_cost: {}",
            pos, min_node, min_node_h_cost, min_code_g_cost
        );

        while pos != self.origin {
            let game_pos = attempt_position_reconstruction(&pos);

            if let Some(pos_at_dest) = path.get_mut(ii) {
                *pos_at_dest = game_pos
            } else {
                path.push(game_pos);
            }

            ii += 1;

            let index = self.parents[index as usize];
            info!("Index: {}", index);
            let next = self.pos_from_index(index);
            if next.range_to(&pos) > 1 {
                let dir = pos.direction_to(&next);

                info!("Attempting serialization from {:?} to {:?} by {}", pos, next, dir);
                let igp = attempt_position_reconstruction(&pos);
                info!("IGP: {:?}", igp);

                loop {
                    if pos.range_to(&next) > 1 {
                        info!("Pos from {:?} to {:?}", pos, pos.position_to_dir(dir));
                        pos = pos.position_to_dir(dir);
                        let game_pos = attempt_position_reconstruction(&pos);

                        if let Some(pos_at_dest) = path.get_mut(ii) {
                            *pos_at_dest = game_pos
                        } else {
                            path.push(game_pos);
                        }

                        ii += 1;
                    } else {
                        break;
                    }
                }
            }

            pos = next;
        }

        PathFinderSearchResult {
            path,
            cost: min_code_g_cost,
            ops: self.max_ops - ops_remaining,
            incomplete: min_node_h_cost != 0,
        }
    }

    pub fn pos_from_index(&self, index: u32) -> WorldPosition {
        let room_index = index / (50 * 50);
        if let Some(info) = &self.room_table[room_index as usize] {
            let coord = index - room_index * 50 * 50;

            world_position(
                coord / 50 + info.pos.xx as u32 * 50,
                coord % 50 + info.pos.yy as u32 * 50,
            )
        } else {
            panic!("Failed to get room info for index: {}", room_index);
        }
    }

    pub fn push_node(&mut self, parent_index: u32, node: WorldPosition, cost: u32) {
        let index = self.index_from_pos(&node);

        if self.open_close.is_closed(index) {
            return;
        }

        let h_cost = (self.heuristic(&node) as f32 * self.heuristic_weight) as u32;
        let f_cost = h_cost + cost as u32;

        if self.open_close.is_open(index) {
            if self.heap.priority(index as usize) > f_cost {
                self.heap.update(index, f_cost);
                self.parents[index as usize] = parent_index;
            }
        } else {
            self.heap.insert(index, f_cost);
            self.open_close.open(index);
            self.parents[index as usize] = parent_index;
        }
    }

    pub fn look(&mut self, pos: &WorldPosition) -> u32 {
        let room_index = self.room_index_from_pos(pos.map_position());
        if room_index == 0 {
            return u32::MAX;
        }

        if let Some(room) = &self.room_table.get_mut((room_index - 1) as usize) {
            if let Some(room) = room {
                let xy = RoomXY::new(
                    RoomCoordinate::new((pos.xx % 50).try_into().unwrap()).unwrap(),
                    RoomCoordinate::new((pos.yy % 50).try_into().unwrap()).unwrap(),
                );
                let terrain = room.terrain.get_xy(xy);
    
                if terrain != Terrain::Plain {
                    if terrain == Terrain::Wall {
                        return u32::MAX;
                    } else {
                        return terrain as u32;
                    }
                }

                room.clone().look((pos.xx % 50) as u8, (pos.yy % 50) as u8) as u32
            } else {
                panic!("Failed to get room info for index: {}", room_index);
            }
        } else {
            panic!("Failed to get room info for index: {}", room_index);
        }
    }

    pub fn astar(&mut self, index: u32, pos: &WorldPosition, g_cost: u32) {
        for dir_num in Direction::Top as u8..Direction::TopLeft as u8 {
            let dir = num_to_dir(dir_num);

            let neighbor = pos.position_to_dir(dir);

            if pos.xx % 50 == 0 {
                if neighbor.xx % 50 == 49 && pos.yy != neighbor.yy {
                    continue;
                } else if pos.xx == neighbor.xx {
                    continue;
                }
            } else if pos.xx % 50 == 49 {
                if neighbor.xx % 50 == 0 && pos.yy != neighbor.yy {
                    continue;
                } else if pos.xx == neighbor.xx {
                    continue;
                }
            } else if pos.yy % 50 == 0 {
                if neighbor.yy % 50 == 49 && pos.xx != neighbor.xx {
                    continue;
                } else if pos.yy == neighbor.yy {
                    continue;
                }
            } else if pos.yy % 50 == 49 {
                if neighbor.yy % 50 == 0 && pos.xx != neighbor.xx {
                    continue;
                } else if pos.yy == neighbor.yy {
                    continue;
                }
            }

            let n_cost = self.look(&neighbor);

            if n_cost == u32::MAX {
                continue;
            }

            self.push_node(index, neighbor, g_cost + n_cost);
        }
    }

    pub fn index_from_pos(&mut self, pos: &WorldPosition) -> u32 {
        let index = self.room_index_from_pos(pos.map_position());
        if index == 0 {
            return 0;
        }
        (index - 1) * 50 * 50 + pos.xx % 50 * 50 + pos.yy % 50
    }

    pub fn room_index_from_pos(&mut self, pos: MapPosition) -> u32 {
        let room_index = self.reverse_room_table[(pos.id() as u32) as usize];

        if room_index == 0 {
            if self.room_table_size >= self.max_rooms as u32 {
                return 0;
            }

            let room_name = RoomName::new(&generate_room_name(pos.xx, pos.yy)).unwrap();
            let terrain = get_room_terrain(room_name);
            if terrain.is_none() {
                panic!("Failed to get terrain for room {}", room_name);
            }

            let map = (self.room_callback)(room_name);

            let id = pos.id();

            self.room_table[self.room_table_size as usize] = Some(RoomInfo {
                terrain: terrain.unwrap(),
                cost_matrix: map,
                pos,
            });
            self.room_table_size += 1;

            self.reverse_room_table[id as usize] = self.room_table_size;

            return self.reverse_room_table[id as usize];
        }
        return room_index;
    }

    pub fn heuristic(&self, pos: &WorldPosition) -> u32 {
        if self.flee {
            let mut ret = 0;
            for goal in &self.goals {
                let distance = pos.range_to(&goal.pos);
                if distance < goal.range {
                    ret = std::cmp::max(ret, goal.range - distance);
                }
            }

            ret
        } else {
            let mut ret = u32::MAX;
            for goal in &self.goals {
                let distance = pos.range_to(&goal.pos);
                if distance > goal.range {
                    ret = std::cmp::min(ret, distance - goal.range);
                } else {
                    ret = 0;
                }
            }

            ret
        }
    }

    pub fn jump_point_search(&mut self, index: u32, pos: WorldPosition, g_cost: u32) {
        let parent = self.pos_from_index(self.parents[index as usize]);
        let dx = if pos.xx > parent.xx {
            1 as i32
        } else if pos.xx < parent.xx {
            -1
        } else {
            0
        };
        let dy = if pos.yy > parent.yy {
            1
        } else if pos.yy < parent.yy {
            -1 as i32
        } else {
            0
        };

        let mut neighbors = [null_world_pos(); 3];
        let mut neighbor_count = 0;
        if pos.xx % 50 == 0 {
            if dx == -1 {
                neighbors[0] = world_position(pos.xx - 1, pos.yy);
                neighbor_count = 1;
            } else if dx == 1 {
                neighbors[0] = world_position(pos.xx + 1, pos.yy - 1);
                neighbors[1] = world_position(pos.xx + 1, pos.yy);
                neighbors[2] = world_position(pos.xx + 1, pos.yy + 1);
                neighbor_count = 3;
            }
        } else if pos.xx % 50 == 49 {
            if dx == 1 {
                neighbors[0] = world_position(pos.xx + 1, pos.yy);
                neighbor_count = 1;
            } else if dx == -1 {
                neighbors[0] = world_position(pos.xx - 1, pos.yy - 1);
                neighbors[1] = world_position(pos.xx - 1, pos.yy);
                neighbors[2] = world_position(pos.xx - 1, pos.yy + 1);
                neighbor_count = 3;
            }
        } else if pos.yy % 50 == 0 {
            if dy == -1 {
                neighbors[0] = world_position(pos.xx, pos.yy - 1);
                neighbor_count = 1;
            } else if dy == 1 {
                neighbors[0] = world_position(pos.xx - 1, pos.yy + 1);
                neighbors[1] = world_position(pos.xx, pos.yy + 1);
                neighbors[2] = world_position(pos.xx + 1, pos.yy + 1);
                neighbor_count = 3;
            }
        } else if pos.yy % 50 == 49 {
            if dy == 1 {
                neighbors[0] = world_position(pos.xx, pos.yy + 1);
                neighbor_count = 1;
            } else if dy == -1 {
                neighbors[0] = world_position(pos.xx - 1, pos.yy - 1);
                neighbors[1] = world_position(pos.xx, pos.yy - 1);
                neighbors[2] = world_position(pos.xx + 1, pos.yy - 1);
                neighbor_count = 3;
            }
        }

        if neighbor_count != 0 {
            for neighbor in neighbors {
                let n_cost = self.look(&neighbor);
                if n_cost == u32::MAX {
                    continue;
                }

                self.push_node(index, neighbor, g_cost + n_cost);
            }

            return;
        }

        let mut border_dx = 0;
        if pos.xx % 50 == 1 {
            border_dx = -1;
        } else if pos.xx % 50 == 48 {
            border_dx = 1;
        }

        let mut border_dy = 0;
        if pos.yy % 50 == 1 {
            border_dy = -1;
        } else if pos.yy % 50 == 48 {
            border_dy = 1;
        }

        let cost = self.look(&pos);
        if dx != 0 {
            let mut neighbor = world_position((pos.xx as i32 + dx) as u32, pos.yy);
            let n_cost = self.look(&neighbor);
            if n_cost != u32::MAX {
                if border_dy == 0 {
                    self.jump_neighbor(&pos, index, &mut neighbor, g_cost, cost, n_cost);
                } else {
                    self.push_node(index, neighbor, g_cost + n_cost);
                }
            }
        }

        if dy != 0 {
            let mut neighbor = world_position(pos.xx, (pos.yy as i32 + dy) as u32);
            let n_cost = self.look(&neighbor);
            if n_cost != u32::MAX {
                if border_dx == 0 {
                    self.jump_neighbor(&pos, index, &mut neighbor, g_cost, cost, n_cost);
                } else {
                    self.push_node(index, neighbor, g_cost + n_cost);
                }
            }
        }

        if dx != 0 {
            if dy != 0 {
                let mut neighbor =
                    world_position((pos.xx as i32 + dx) as u32, (pos.yy as i32 + dy) as u32);
                let n_cost = self.look(&neighbor);
                if n_cost != u32::MAX {
                    self.jump_neighbor(&pos, index, &mut neighbor, g_cost, cost, n_cost);
                }
                if self.look(&world_position((pos.xx as i32 - dx) as u32, pos.yy)) != cost {
                    let look = self.look(&world_position(
                        (pos.xx as i32 - dx) as u32,
                        (pos.yy as i32 + dy) as u32,
                    ));
                    self.jump_neighbor(
                        &pos,
                        index,
                        &mut world_position(
                            (pos.xx as i32 - dx) as u32,
                            (pos.yy as i32 + dy) as u32,
                        ),
                        g_cost,
                        cost,
                        look,
                    );
                }
                if self.look(&world_position(pos.xx, (pos.yy as i32 - dy) as u32)) != cost {
                    let look = self.look(&world_position(
                        (pos.xx as i32 - dx) as u32,
                        (pos.yy as i32 + dy) as u32,
                    ));
                    self.jump_neighbor(
                        &pos,
                        index,
                        &mut world_position(
                            (pos.xx as i32 - dx) as u32,
                            (pos.yy as i32 + dy) as u32,
                        ),
                        g_cost,
                        cost,
                        look,
                    );
                }
            } else {
                if border_dy == 1 || self.look(&world_position(pos.xx, pos.yy + 1)) != cost {
                    let look = self.look(&world_position((pos.xx as i32 + dx) as u32, pos.yy + 1));
                    self.jump_neighbor(
                        &pos,
                        index,
                        &mut world_position((pos.xx as i32 + dx) as u32, pos.yy + 1),
                        g_cost,
                        cost,
                        look,
                    );
                }
                if border_dy == -1 || self.look(&world_position(pos.xx, pos.yy - 1)) != cost {
                    let look = self.look(&world_position((pos.xx as i32 + dx) as u32, pos.yy - 1));
                    self.jump_neighbor(
                        &pos,
                        index,
                        &mut world_position((pos.xx as i32 + dx) as u32, pos.yy - 1),
                        g_cost,
                        cost,
                        look,
                    );
                }
            }
        } else {
            if border_dx == 1 || self.look(&world_position(pos.xx + 1, pos.yy)) != cost {
                let look = self.look(&world_position(pos.xx + 1, (pos.yy as i32 + dy) as u32));
                self.jump_neighbor(
                    &pos,
                    index,
                    &mut world_position(pos.xx + 1, (pos.yy as i32 + dy) as u32),
                    g_cost,
                    cost,
                    look,
                );
            }
            if border_dx == -1 || self.look(&world_position(pos.xx - 1, pos.yy)) != cost {
                let look = self.look(&world_position(pos.xx - 1, (pos.yy as i32 + dy) as u32));
                self.jump_neighbor(
                    &pos,
                    index,
                    &mut world_position(pos.xx - 1, (pos.yy as i32 + dy) as u32),
                    g_cost,
                    cost,
                    look,
                );
            }
        }
    }

    pub fn jump_neighbor(
        &mut self,
        pos: &WorldPosition,
        index: u32,
        neighbor: &mut WorldPosition,
        mut g_cost: u32,
        cost: u32,
        n_cost: u32,
    ) {
        if n_cost != cost || is_border_pos(neighbor.xx) || is_border_pos(neighbor.yy) {
            if n_cost == u32::MAX {
                return;
            }
            g_cost += n_cost;
        } else {
            let neighbor = self.jump(
                n_cost,
                neighbor,
                (neighbor.xx - pos.xx) as i32,
                (neighbor.yy - pos.yy) as i32,
            );
            if neighbor.is_null() {
                return;
            }
            g_cost += n_cost * (pos.range_to(&neighbor) - 1) + self.look(&neighbor);
        }

        self.push_node(index, *neighbor, g_cost);
    }

    pub fn jump(&mut self, cost: u32, pos: &mut WorldPosition, dx: i32, dy: i32) -> WorldPosition {
        if dx != 0 {
            if dy != 0 {
                return self.jump_xy(cost, pos, dx, dy);
            } else {
                return self.jump_x(cost, pos, dx);
            }
        } else {
            return self.jump_y(cost, pos, dy);
        }
    }

    pub fn jump_xy(
        &mut self,
        cost: u32,
        mut pos: &mut WorldPosition,
        dx: i32,
        dy: i32,
    ) -> WorldPosition {
        let mut prev_cost_x = self.look(&world_position((pos.xx as i32 - dx) as u32, pos.yy));
        let mut prev_cost_y = self.look(&world_position(pos.xx, pos.yy - dy as u32));

        loop {
            if self.heuristic(&pos) == 0 || is_near_border_pos(pos.xx) || is_near_border_pos(pos.yy)
            {
                break;
            }

            if self.look(&world_position(
                (pos.xx as i32 - dx) as u32,
                (pos.yy as i32 + dy) as u32,
            )) != u32::MAX
                && prev_cost_x != cost
                || self.look(&world_position(
                    (pos.xx as i32 + dx) as u32,
                    (pos.yy as i32 - dy) as u32,
                )) != u32::MAX
                    && prev_cost_y != cost
            {
                break;
            }

            prev_cost_x = self.look(&world_position(pos.xx, (pos.yy as i32 + dy) as u32));
            prev_cost_y = self.look(&world_position((pos.xx as i32 + dx) as u32, pos.yy));

            if prev_cost_y != u32::MAX
                && !self
                    .jump_x(
                        cost,
                        &mut world_position((pos.xx as i32 + dx) as u32, pos.yy),
                        dx,
                    )
                    .is_null()
                || prev_cost_x != u32::MAX
                    && !self
                        .jump_y(
                            cost,
                            &mut world_position(pos.xx, (pos.yy as i32 + dy) as u32),
                            dy,
                        )
                        .is_null()
            {
                break;
            }

            pos.xx += dx as u32;
            pos.yy += dy as u32;

            let jump_cost = self.look(&pos);
            if jump_cost == u32::MAX {
                return null_world_pos();
                break;
            } else if jump_cost != cost {
                break;
            }
        }

        return pos.clone();
    }

    pub fn jump_x(&mut self, cost: u32, pos: &mut WorldPosition, dx: i32) -> WorldPosition {
        let mut prev_cost_u = self.look(&world_position(pos.xx, pos.yy - 1));
        let mut prev_cost_d = self.look(&world_position(pos.xx, pos.yy + 1));

        loop {
            if self.heuristic(&pos) == 0 || is_near_border_pos(pos.xx) {
                break;
            }

            let cost_u = self.look(&world_position(pos.xx, pos.yy - 1));
            let cost_d = self.look(&world_position(pos.xx, pos.yy + 1));
            if cost_u != u32::MAX && prev_cost_u != cost
                || cost_d != u32::MAX && prev_cost_d != cost
            {
                break;
            }

            prev_cost_u = cost_u;
            prev_cost_d = cost_d;
            pos.xx += dx as u32;

            let jump_cost = self.look(&pos);
            if jump_cost == u32::MAX {
                return null_world_pos();
            } else if jump_cost != cost {
                break;
            }
        }

        return pos.clone();
    }

    pub fn jump_y(&mut self, cost: u32, pos: &mut WorldPosition, dy: i32) -> WorldPosition {
        let mut prev_cost_l = self.look(&world_position(pos.xx - 1, pos.yy));
        let mut prev_cost_r = self.look(&world_position(pos.xx + 1, pos.yy));

        loop {
            if self.heuristic(&pos) == 0 || is_near_border_pos(pos.yy) {
                break;
            }

            let cost_l = self.look(&world_position(pos.xx - 1, (pos.yy as i32 + dy) as u32));
            let cost_r = self.look(&world_position(pos.xx + 1, (pos.yy as i32 + dy) as u32));
            if cost_l != u32::MAX && prev_cost_l != cost
                || cost_r != u32::MAX && prev_cost_r != cost
            {
                break;
            }

            prev_cost_l = cost_l;
            prev_cost_r = cost_r;
            pos.yy += dy as u32;

            let jump_cost = self.look(&pos);
            if jump_cost == u32::MAX {
                return null_world_pos();
            } else if jump_cost != cost {
                break;
            }
        }

        return pos.clone();
    }
}

pub fn is_border_pos(pos: u32) -> bool {
    return (pos + 1) % 50 < 2;
}

pub fn is_near_border_pos(pos: u32) -> bool {
    return (pos + 2) % 50 < 4;
}

pub fn get_room_terrain(room: RoomName) -> Option<LocalRoomTerrain> {
    if let Some(terrain) = game::map::get_room_terrain(room) {
        return Some(LocalRoomTerrain::from(terrain));
    } else {
        return None;
    }
}

pub fn attempt_position_reconstruction(pos: &WorldPosition) -> Position {
    let x = RoomCoordinate::new((pos.xx % 50) as u8).unwrap();
    let y = RoomCoordinate::new((pos.yy % 50) as u8).unwrap();
    let room_name = generate_room_name((pos.xx as f32 / 50.0).floor() as u8, (pos.yy as f32 / 50.0).floor() as u8);

    let game_pos = Position::new(x, y, RoomName::new(&room_name).unwrap());

    game_pos
}