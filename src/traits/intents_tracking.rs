#![allow(non_snake_case)]
#![allow(unused)]
use js_sys::{Array, JsString};
use screeps::{Attackable, Color, ConstructionSite, Creep, Direction, Dismantleable, ErrorCode, Harvestable, Healable, Repairable, Resource, ResourceType, Room, RoomName, RoomPosition, SharedCreepProperties, SpawnOptions, Structure, StructureController, StructureFactory, StructureLab, StructureLink, StructureNuker, StructureObject, StructureObserver, StructurePowerSpawn, StructureProperties, StructureRampart, StructureTower, StructureType, Transferable, Withdrawable};

use crate::profiling::timing::INTENTS_USED;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
pub fn track_intent() {
    let mut count = INTENTS_USED.lock().unwrap();

    *count += 1;
}

pub trait ConstructionExtensionsTracking {
    fn ITremove(&self) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl ConstructionExtensionsTracking for ConstructionSite {
    fn ITremove(&self) -> Result<(), ErrorCode> {
        track_intent();
        self.remove()
    }
}

pub trait FlagExtensionsTracking {
    fn ITremove(&self);
    fn ITset_color(&self, color: Color, secondary_color: Option<Color>);
    fn ITset_position(&self, position: RoomPosition);
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl FlagExtensionsTracking for screeps::Flag {
    fn ITremove(&self) {
        track_intent();
        self.remove();
    }

    fn ITset_color(&self, color: Color, secondary_color: Option<Color>) {
        track_intent();
        self.set_color(color, secondary_color);
    }

    fn ITset_position(&self, position: RoomPosition) {
        track_intent();
        self.set_position(position);
    }
}

pub trait CreepExtensionsTracking {
    fn ITmove_direction(&self, dir: Direction)-> Result<(), ErrorCode>;
    fn ITattack(&self, target: &dyn Attackable) -> Result<(), ErrorCode>;
    fn ITattack_controller(&self, target: &StructureController) -> Result<(), ErrorCode>;
    fn ITbuild(&self, target: &ConstructionSite) -> Result<(), ErrorCode>;
    fn ITclaim_controller(&self, target: &StructureController) -> Result<(), ErrorCode>;
    fn ITdismantle(&self, target: &dyn Dismantleable) -> Result<(), ErrorCode>;
    fn ITdrop(&self, resource_type: ResourceType, amount: Option<u32>) -> Result<(), ErrorCode>;
    fn ITgenerate_safe_mode(&self, controller: &StructureController) -> Result<(), ErrorCode>;
    fn ITharvest(&self, source: &dyn Harvestable) -> Result<(), ErrorCode>;
    fn ITheal(&self, target: &dyn Healable) -> Result<(), ErrorCode>;
    fn ITpickup(&self, target: &Resource) -> Result<(), ErrorCode>;
    fn ITranged_attack(&self, target: &dyn Attackable) -> Result<(), ErrorCode>;
    fn ITranged_heal(&self, target: &dyn Healable) -> Result<(), ErrorCode>;
    fn ITranged_mass_attack(&self) -> Result<(), ErrorCode>;
    fn ITrepair(&self, target: &dyn Repairable) -> Result<(), ErrorCode>;
    fn ITreserve_controller(&self, target: &StructureController) -> Result<(), ErrorCode>;
    fn ITsign_controller(&self, target: &StructureController, text: &str) -> Result<(), ErrorCode>;
    fn ITsuicide(&self) -> Result<(), ErrorCode>;
    fn ITtransfer(&self, target: &dyn Transferable, resource_type: ResourceType, amount: Option<u32>) -> Result<(), ErrorCode>;
    fn ITupgrade_controller(&self, target: &StructureController) -> Result<(), ErrorCode>;
    fn ITwithdraw(&self, target: &dyn Withdrawable, resource_type: ResourceType, amount: Option<u32>) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl CreepExtensionsTracking for Creep {
    fn ITmove_direction(&self, dir: Direction) -> Result<(), ErrorCode> {
        track_intent();
        self.move_direction(dir)
    }

    fn ITattack(&self, target: &dyn Attackable) -> Result<(), ErrorCode> {
        track_intent();
        self.attack(target)
    }

    fn ITattack_controller(&self, target: &StructureController) -> Result<(), ErrorCode> {
        track_intent();
        self.attack_controller(target)
    }

    fn ITbuild(&self, target: &ConstructionSite) -> Result<(), ErrorCode> {
        track_intent();
        self.build(target)
    }

    fn ITclaim_controller(&self, target: &StructureController) -> Result<(), ErrorCode> {
        track_intent();
        self.claim_controller(target)
    }

    fn ITdismantle(&self, target: &dyn Dismantleable) -> Result<(), ErrorCode> {
        track_intent();
        self.dismantle(target)
    }

    fn ITdrop(&self, resource_type: ResourceType, amount: Option<u32>) -> Result<(), ErrorCode> {
        track_intent();
        self.drop(resource_type, amount)
    }

    fn ITgenerate_safe_mode(&self, controller: &StructureController) -> Result<(), ErrorCode> {
        track_intent();
        self.generate_safe_mode(controller)
    }

    fn ITharvest(&self, source: &dyn Harvestable) -> Result<(), ErrorCode> {
        track_intent();
        self.harvest(source)
    }

    fn ITheal(&self, target: &dyn Healable) -> Result<(), ErrorCode> {
        track_intent();
        self.heal(target)
    }

    fn ITpickup(&self, target: &Resource) -> Result<(), ErrorCode> {
        track_intent();
        self.pickup(target)
    }

    fn ITranged_attack(&self, target: &dyn Attackable) -> Result<(), ErrorCode> {
        track_intent();
        self.ranged_attack(target)
    }

    fn ITranged_heal(&self, target: &dyn Healable) -> Result<(), ErrorCode> {
        track_intent();
        self.ranged_heal(target)
    }

    fn ITranged_mass_attack(&self) -> Result<(), ErrorCode> {
        track_intent();
        self.ranged_mass_attack()
    }

    fn ITrepair(&self, target: &dyn Repairable) -> Result<(), ErrorCode> {
        track_intent();
        self.repair(target)
    }

    fn ITreserve_controller(&self, target: &StructureController) -> Result<(), ErrorCode> {
        track_intent();
        self.reserve_controller(target)
    }

    fn ITsign_controller(&self, target: &StructureController, text: &str) -> Result<(), ErrorCode> {
        track_intent();
        self.sign_controller(target, text)
    }

    fn ITsuicide(&self) -> Result<(), ErrorCode> {
        track_intent();
        self.suicide()
    }

    fn ITtransfer(&self, target: &dyn Transferable, resource_type: ResourceType, amount: Option<u32>) -> Result<(), ErrorCode> {
        track_intent();
        self.transfer(target, resource_type, amount)
    }

    fn ITupgrade_controller(&self, target: &StructureController) -> Result<(), ErrorCode> {
        track_intent();
        self.upgrade_controller(target)
    }

    fn ITwithdraw(&self, target: &dyn Withdrawable, resource_type: ResourceType, amount: Option<u32>) -> Result<(), ErrorCode> {
        track_intent();
        self.withdraw(target, resource_type, amount)
    }
}

pub trait RoomExtensionsTracking {
    fn ITcreate_construction_site(&self, x: u8, y: u8, structure_type: StructureType, name: Option<&js_sys::JsString>) -> Result<(), ErrorCode>;
    fn ITcreate_flag(&self, x: u8, y: u8, name: Option<&js_sys::JsString>, color: Option<Color>, secondary_color: Option<Color>) -> Result<JsString, ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomExtensionsTracking for Room {
    fn ITcreate_construction_site(&self, x: u8, y: u8, structure_type: StructureType, name: Option<&js_sys::JsString>) -> Result<(), ErrorCode> {
        track_intent();
        self.create_construction_site(x, y, structure_type, name)
    }

    fn ITcreate_flag(&self, x: u8, y: u8, name: Option<&js_sys::JsString>, color: Option<Color>, secondary_color: Option<Color>) -> Result<JsString, ErrorCode> {
        track_intent();
        self.create_flag(x, y, name, color, secondary_color)
    }
}

pub trait RoomPositionExtensionsTracking {
    fn ITcreate_construction_site(&self, structure_type: StructureType, name: Option<&js_sys::JsString>) -> Result<(), ErrorCode>;
    fn ITcreate_flag(&self, name: Option<&js_sys::JsString>, color: Option<Color>, secondary_color: Option<Color>) -> Result<JsString, ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl RoomPositionExtensionsTracking for RoomPosition {
    fn ITcreate_construction_site(&self, structure_type: StructureType, name: Option<&js_sys::JsString>) -> Result<(), ErrorCode> {
        track_intent();
        self.create_construction_site(structure_type, name)
    }

    fn ITcreate_flag(&self, name: Option<&js_sys::JsString>, color: Option<Color>, secondary_color: Option<Color>) -> Result<JsString, ErrorCode> {
        track_intent();
        self.create_flag(name, color, secondary_color)
    }
}

pub trait StructureExtensionsTracking {
    fn ITdestroy(&self) -> i8;
    fn ITnotify_when_attacked(&self, enabled: bool) -> i8;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureExtensionsTracking for Structure {
    fn ITdestroy(&self) -> i8 {
        track_intent();
        self.destroy()
    }

    fn ITnotify_when_attacked(&self, enabled: bool) -> i8 {
        track_intent();
        self.notify_when_attacked(enabled)
    }
}

pub trait StructureObjectTracking {
    fn ITdestroy(&self) -> Result<(), ErrorCode>;
    fn ITnotify_when_attacked(&self, enabled: bool) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureObjectTracking for StructureObject {
    fn ITdestroy(&self) -> Result<(), ErrorCode> {
        track_intent();
        self.destroy()
    }

    fn ITnotify_when_attacked(&self, enabled: bool) -> Result<(), ErrorCode> {
        track_intent();
        self.notify_when_attacked(enabled)
    }
}

pub trait StructureControllerExtensionsTracking {
    fn ITactivate_safe_mode(&self) -> Result<(), ErrorCode>;
    fn ITunclaim(&self) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureControllerExtensionsTracking for StructureController {
    fn ITactivate_safe_mode(&self) -> Result<(), ErrorCode> {
        track_intent();
        self.activate_safe_mode()
    }

    fn ITunclaim(&self) -> Result<(), ErrorCode> {
        track_intent();
        self.unclaim()
    }
}

pub trait StructureFactoryExtensionsTracking {
    fn ITproduce(&self, resource_type: ResourceType) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureFactoryExtensionsTracking for StructureFactory {
    fn ITproduce(&self, resource_type: ResourceType) -> Result<(), ErrorCode> {
        track_intent();
        self.produce(resource_type)
    }
}

pub trait StructureLabExtensionsTracking {
    fn ITboost_creep(&self, target: &Creep, body_parts_count: Option<u32>) -> Result<(), ErrorCode>;
    fn ITreverse_reaction(&self, lab1: &StructureLab, lab2: &StructureLab) -> Result<(), ErrorCode>;
    fn ITrun_reaction(&self, lab1: &StructureLab, lab2: &StructureLab) -> Result<(), ErrorCode>;
    fn ITunboost_creep(&self, target: &Creep) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureLabExtensionsTracking for StructureLab {
    fn ITboost_creep(&self, target: &Creep, body_parts_count: Option<u32>) -> Result<(), ErrorCode> {
        track_intent();
        self.boost_creep(target, body_parts_count)
    }

    fn ITreverse_reaction(&self, lab1: &StructureLab, lab2: &StructureLab) -> Result<(), ErrorCode> {
        track_intent();
        self.reverse_reaction(lab1, lab2)
    }

    fn ITrun_reaction(&self, lab1: &StructureLab, lab2: &StructureLab) -> Result<(), ErrorCode> {
        track_intent();
        self.run_reaction(lab1, lab2)
    }

    fn ITunboost_creep(&self, target: &Creep) -> Result<(), ErrorCode> {
        track_intent();
        self.unboost_creep(target)
    }
}

pub trait StructureLinkExtensionsTracking {
    fn ITtransfer_energy(&self, target: &StructureLink, amount: Option<u32>) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureLinkExtensionsTracking for StructureLink {
    fn ITtransfer_energy(&self, target: &StructureLink, amount: Option<u32>) -> Result<(), ErrorCode> {
        track_intent();
        self.transfer_energy(target, amount)
    }
}

pub trait StructureNukerExtensionsTracking {
    fn ITlaunch_nuke(&self, target: &RoomPosition) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureNukerExtensionsTracking for StructureNuker {
    fn ITlaunch_nuke(&self, target: &RoomPosition) -> Result<(), ErrorCode> {
        track_intent();
        self.launch_nuke(target)
    }
}

pub trait StructureObserverExtensionsTracking {
    fn ITobserve_room(&self, room: RoomName) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureObserverExtensionsTracking for StructureObserver {
    fn ITobserve_room(&self, room: RoomName) -> Result<(), ErrorCode> {
        track_intent();
        self.observe_room(room)
    }
}

pub trait StructurePowerSpawnExtensionsTracking {
    fn ITprocess_power(&self) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructurePowerSpawnExtensionsTracking for StructurePowerSpawn {
    fn ITprocess_power(&self) -> Result<(), ErrorCode> {
        track_intent();
        self.process_power()
    }
}

pub trait StructureRampartExtensionsTracking {
    fn ITset_public(&self, is_public: bool) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureRampartExtensionsTracking for StructureRampart {
    fn ITset_public(&self, is_public: bool) -> Result<(), ErrorCode> {
        track_intent();
        self.set_public(is_public)
    }
}

pub trait StructureSpawnExtensionsTracking {
    fn ITspawn_creep(&self, body: &[screeps::Part], name: &str) -> Result<(), ErrorCode>;
    fn ITspawn_creep_with_options(&self, body: &[screeps::Part], name: &str, options: &SpawnOptions) -> Result<(), ErrorCode>;
    fn ITrenew_creep(&self, target: &Creep) -> Result<(), ErrorCode>;
    fn ITrecycle_creep(&self, target: &Creep) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureSpawnExtensionsTracking for screeps::StructureSpawn {
    fn ITspawn_creep(&self, body: &[screeps::Part], name: &str) -> Result<(), ErrorCode> {
        track_intent();
        self.spawn_creep(body, name)
    }

    fn ITspawn_creep_with_options(&self, body: &[screeps::Part], name: &str, options: &SpawnOptions) -> Result<(), ErrorCode> {
        track_intent();
        self.spawn_creep_with_options(body, name, options)
    }

    fn ITrenew_creep(&self, target: &Creep) -> Result<(), ErrorCode> {
        track_intent();
        self.renew_creep(target)
    }

    fn ITrecycle_creep(&self, target: &Creep) -> Result<(), ErrorCode> {
        track_intent();
        self.recycle_creep(target)
    }
}

pub trait SpawningExtensionsTracking {
    fn ITcancel(&self) -> Result<(), ErrorCode>;
    fn ITset_directions(&self, directions: &Array) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl SpawningExtensionsTracking for screeps::Spawning {
    fn ITcancel(&self) -> Result<(), ErrorCode> {
        track_intent();
        self.cancel()
    }

    fn ITset_directions(&self, directions: &Array) -> Result<(), ErrorCode> {
        track_intent();
        self.set_directions(directions)
    }
}

pub trait StructureTerminalExtensionsTracking {
    fn ITsend(&self, resource_type: ResourceType, amount: u32, destination: RoomName, description: Option<&str>) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl StructureTerminalExtensionsTracking for screeps::StructureTerminal {
    fn ITsend(&self, resource_type: ResourceType, amount: u32, destination: RoomName, description: Option<&str>) -> Result<(), ErrorCode> {
        track_intent();
        self.send(resource_type, amount, destination, description)
    }
}

pub trait TowerExtensionsTracking {
    fn ITattack(&self, target: &dyn Attackable) -> Result<(), ErrorCode>;
    fn ITheal(&self, target: &dyn Healable) -> Result<(), ErrorCode>;
    fn ITrepair(&self, target: &dyn Repairable) -> Result<(), ErrorCode>;
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl TowerExtensionsTracking for StructureTower {
    fn ITattack(&self, target: &dyn Attackable) -> Result<(), ErrorCode> {
        track_intent();
        self.attack(target)
    }

    fn ITheal(&self, target: &dyn Healable) -> Result<(), ErrorCode> {
        track_intent();
        self.heal(target)
    }

    fn ITrepair(&self, target: &dyn Repairable) -> Result<(), ErrorCode> {
        track_intent();
        self.repair(target)
    }
}