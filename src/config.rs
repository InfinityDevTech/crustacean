pub const MEMORY_VERSION: u8 = 1;

// Hate weights, determins how much weight is adder per event.
pub const HATE_CREEP_ATTACK_WEIGHT: f32 = 1.0;
pub const HATE_CREEP_HEAL_WEIGHT: f32 = -0.5;

// Hate decay rate, how much hate is lost per tick.
pub const HATE_DECAY_RATE: f32 = 0.5;
pub const TICKS_BEFORE_DECAY: u32 = 500;