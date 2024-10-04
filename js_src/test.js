export function do_classify_find(room_name) {
    let room = Game.rooms[room_name];
  
    if (room) {
      let res = room.find(FIND_STRUCTURES);
  
      let all_structures = [];
      let repairable = [];
      let container = [];
      let link = [];
  
      res.forEach((structure) => {
        all_structures.push(structure);

        //let can_own = switch(structure.structureType) {
        //  case:
        //}
  
        if (structure.hits < structure.hitsMax && structure.structureType != STRUCTURE_WALL) {
          repairable.push(structure)
        }
  
        if (structure.structureType == STRUCTURE_LINK) {
          link.push(structure)
        }
  
        if (structure.structureType == STRUCTURE_CONTAINER) {
          container.push(structure);
        }
      })
  
      return [all_structures, repairable, container, link];
    } else {
      return [];
    }
  }