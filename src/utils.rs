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

pub fn name_to_role(name: &str) -> Option<Role> {
    match &name[..2] {
        "sm" => { Some(Role::Miner) },
        "mb" => { Some(Role::Hauler) },
        "ud" => { Some(Role::Upgrader) },
        "bd" => { Some(Role::Builder) },
        "fg" => { Some(Role::Scout) },
        _ => { None },
    }
}