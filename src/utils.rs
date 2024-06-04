use crate::{memory::Role, room::cache::tick_cache::hauling::HaulingPriority};

/// Scale the hauling priority based on the amount of resources in the target.
/// Capacity: The total capacity of the target
/// Amount: The amount of resources in the target
/// Priority: The priority of the task
/// Reverse: Get priority based off of how empty the container is
pub fn scale_haul_priority(capacity: u32, amount: u32, priority: HaulingPriority, reverse: bool) -> u32 {
    let priority = priority as u32;

    if capacity == 0 {
        return 0;
    }

    if reverse {
        return (1 - amount / capacity) * priority;
    }

    (amount / capacity) * priority
}

/// Convert a role to its respective string
/// **Example:** Miner **=** sm
/// **Example:** Hauler **=** mb
pub fn role_to_name(role: Role) -> String {
    let data = match role {
        Role::Miner => "sm",
        Role::Hauler => "mb",
        Role::Upgrader => "ud",
        Role::Builder => "bd",
        Role::Scout => "fg",
        Role::FastFiller => "ff",
        Role::Bulldozer => "sa",
    };
    data.to_string()
}

/// Convert a string to its respective role
/// **Example:** sm **=** Miner
/// **Example:** mb **=** Hauler
pub fn name_to_role(name: &str) -> Option<Role> {
    match &name[..2] {
        "sm" => { Some(Role::Miner) },
        "mb" => { Some(Role::Hauler) },
        "ud" => { Some(Role::Upgrader) },
        "bd" => { Some(Role::Builder) },
        "fg" => { Some(Role::Scout) },
        "ff" => { Some(Role::FastFiller) }
        "sa" => { Some(Role::Bulldozer) },
        _ => { None },
    }
}