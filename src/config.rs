pub const MEMORY_VERSION: u8 = 1;

// Combat event weights
// Affects how the hate of a player grows
const MELEE_ATTACK_WEIGHT: f32 = 1.0;
const RANGED_ATTACK_WEIGHT: f32 = 0.5;
const RANGED_MASS_ATTACK_WEIGHT: f32 = 1.5;
const DISMANTLE_WEIGHT: f32 = 5.0;
const HIT_BACK_WEIGHT: f32 = 0.5;
const NUKE_WEIGHT: f32 = 25.0;

const CONTROLLER_ATTACK: f32 = 40.0;