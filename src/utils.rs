use crate::memory::Role;

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