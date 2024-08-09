use screeps::{game, HasId, StructureObject, StructureTerminal};

use crate::TERMINAL_TO_USE;

pub fn get_terminal() -> Option<StructureTerminal> {
    let mut terminal = TERMINAL_TO_USE.lock().unwrap();

    if let Some(known_terminal) = terminal.as_ref() {
        let game_terminal = game::get_object_by_id_typed(known_terminal);

        if let Some(game_terminal) = game_terminal {
            return Some(game_terminal);
        } else if let Some(found_terminal) = find_terminal() {
            *terminal = Some(found_terminal.id());
            return Some(found_terminal);
        }
    } else if let Some(found_terminal) = find_terminal() {
        *terminal = Some(found_terminal.id());
        return Some(found_terminal);
    }

    None
}

pub fn find_terminal() -> Option<StructureTerminal> {
    for structure in game::structures().values() {
        if let StructureObject::StructureTerminal(terminal) = structure {
            return Some(terminal);
        }
    }

    None
}
