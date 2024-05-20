use crate::memory::Role;

pub fn role_to_name(role: Role) -> String {
    let mut data: &str = "";
    match role {
        Role::Miner => { data = "sm" },
        Role::Hauler => { data = "mb" },
        Role::Upgrader => { data = "ud" },
        Role::Builder => { data = "bd" },
        Role::Scout => { data = "fg" },
    }
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