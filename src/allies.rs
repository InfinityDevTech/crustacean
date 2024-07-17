use screeps::RoomName;

pub fn is_ally(user: &str, room_name: Option<RoomName>) -> bool {
    if let Some(room_name) = room_name {

    }

    let user = user.to_lowercase();

    // This is an operation against all S0 OM users.
    if user == "droidfreak" || user == "v1king" {
        return true;
    }

    // Chill dude, gave me energy on my first S0 room
    if user == "neoncamouflage" {
        return true;
    }

    false
}