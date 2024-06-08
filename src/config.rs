pub const MEMORY_VERSION: u8 = 1;

//pub const ALLIANCE_TAG: &str = "(SSS)";
pub const ALLIANCE_TAG: &str = "";

pub const ALLIES: [&str; 3] = ["MarvinTMB", "Shylo132", "kbharlem"];

pub const ROOM_SIGNS: [&str; 21] = [
//"Rust programming is just crab game.",
//"Web Assembly is overrated",
//"Warning: This room is under the control of an idiot.",
//"Why did the creep cross the road? To escape a tigga quad!",
"Marvin lies!",
//"Kick me.",
"Marx would be dissapointed",
"Made a mess and the war got cold.",
"Bourgeoisie member.",
"The international will be defeated.",
"We must stop Marvin at all costs!",
"Pride of Lenin took Trotsky out of the picture.",
"Stop the revolution!",
"Tore down that wall like the koolaid man.",
"Communism, everyones mortal enemy.",
"Stop the red iceberg!",
"Cease the Collectivization!",
"Its not Communism, its Marvinism!",
"Communism != Collectivization",
"Why did the creeps cross the road. They were under Marvin's collectivized control.",
"Real communism requires individual control.",
"The top 1% dont control as much as Marvin. Stop the collectivization!",
"Marvin is a collectivized menace.",
"Workers of the world, unite! Against Marvin.",
"Screeps bots spend a combined 13 years of ther life under a dictatorship: Marvin.",
"We already eat from the trashcan all the time. The name of this trash is collectivization - Infinity Dev"
];

// Max 10 characters
pub const ATTACK_SIGNS: [&str; 3] = [
    "I <3 U",
    "♾️",
    "🦀",
];

// Hate weights, determins how much weight is adder per event.
pub const HATE_CREEP_ATTACK_WEIGHT: f32 = 1.0;
pub const HATE_CREEP_KILLED_WEIGHT: f32 = 2.0;
pub const HATE_CREEP_HEAL_WEIGHT: f32 = -0.5;

// Hate decay rate, how much hate is lost per tick.
pub const HATE_DECAY_PERCENTEAGE: f32 = 0.99999;
pub const TICKS_BEFORE_DECAY: u32 = 500;