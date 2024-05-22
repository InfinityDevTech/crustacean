use screeps::{Room, RoomName};

use crate::{memory::Role, traits::room::RoomExtensions};

pub fn room_to_number(room: &Room) -> u32 {
    let parts = room.split_name();

    let w_e = match parts.0.as_str() {
        "W" => 1,
        "E" => 2,
        _ => 0,
    };

    let n_s = match parts.2.as_str() {
        "N" => 1,
        "S" => 2,
        _ => 0,
    };

    return w_e + parts.1 + n_s + parts.3;
}

pub fn role_to_name(role: Role) -> String {
    let data = match role {
        Role::Miner => "sm",
        Role::Hauler => "mb",
        Role::Upgrader => "ud",
        Role::Builder => "bd",
        Role::Scout => "fg",
    };
    data.to_string()
}

pub fn name_to_role(name: &str) -> Role {
    match name {
        "sm" => { Role::Miner },
        "mb" => { Role::Hauler },
        "ud" => { Role::Upgrader },
        "bd" => { Role::Builder },
        "fg" => { Role::Scout },
        _ => { Role::Miner },
    }
}