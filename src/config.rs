pub const MEMORY_VERSION: u8 = 1;

pub const VISUALISE_SCOUTING_DATA: bool = false;
//pub const ALLIANCE_TAG: &str = "(SSS)";
pub const ALLIANCE_TAG: &str = "";

pub const ROOM_REMOTE_COUNT: u8 = 8;

pub const INVADER_USERNAME: &str = "Invader";

pub const REMOTE_SIGNS: [&str; 1] = [
//"Woops. - Infinity Dev",
//"Boo. - Infinity Dev",
//"FREE CANDY!!! - Infinity Dev",
//"Infinity wuz here - Infinity Dev",
//"Personally, I think we should delete invaders. Oh! And tigga, because tiger scary. - Infinity Dev",
//"Ourobot V2, more like, 🐍bot. Idk, im just bored at this point. - Infinity Dev",
//"I couldnt think of any other funny names. Just pretend I did. - Infinity Dev",
//"oOoOoOoOoh Taxes. - Infinity Dev",
//"Dont mess with me, my mom called me handsome. - Infinity Dev",
//"I think im a bot? - Infinity Dev",
"liaohao isnt gone. Just ask lp136. - Infinity Dev",
];

pub const ROOM_SIGNS: [&str; 24] = [
//"Rust programming is just crab game.",
//"Web Assembly is overrated",
//"Warning: This room is under the control of an idiot.",
//"Why did the creep cross the road? To escape a tigga quad!",
"Marvin lies!",
//"Kick me.",
"Marx would be dissapointed",
"If your name end with 'in', time to get out.",
"How many dictators does it take to turn an empire into a union of ruinous states?",
"Did somebody say, 'Communism'?",
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
"Why did the creeps cross the road? They were under Marvin's collectivized control.",
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

pub const CREEP_SONG: [&str; 483] = [
"There",
"lived",
"a",
"certain",
"man",
"in",
"Russia",
"long",
"ago",
"He",
"was",
"big",
"and",
"strong,",
"in",
"his",
"eyes",
"a",
"flaming",
"glow",
"Most",
"people",
"look",
"at",
"him",
"with",
"terror",
"and",
"with",
"fear",
"But",
"to",
"Moscow",
"chicks",
"he",
"was",
"such",
"a",
"lovely",
"dear",
"He",
"could",
"preach",
"the",
"Bible",
"like",
"a",
"preacher",
"Full",
"of",
"ecstasy",
"and",
"fire",
"But",
"he",
"also",
"was",
"the",
"kind",
"of",
"teacher",
"Women",
"would",
"desire",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"There",
"lived",
"a",
"certain",
"man",
"in",
"Russia",
"long",
"ago",
"He",
"was",
"big",
"and",
"strong,",
"in",
"his",
"eyes",
"a",
"flaming",
"glow",
"Most",
"people",
"look",
"at",
"him",
"with",
"terror",
"and",
"with",
"fear",
"But",
"to",
"Moscow",
"chicks",
"he",
"was",
"such",
"a",
"lovely",
"dear",
"He",
"could",
"preach",
"the",
"Bible",
"like",
"a",
"preacher",
"Full",
"of",
"ecstasy",
"and",
"fire",
"But",
"he",
"also",
"was",
"the",
"kind",
"of",
"teacher",
"Women",
"would",
"desire",
"Ra-ra-",
"Rasputin",
"Lover",
"of",
"the",
"Russian",
"queen",
"There",
"was",
"a",
"cat",
"that",
"really",
"was",
"gone",
"Ra-ra-",
"Rasputin",
"Russia's",
"greatest",
"love",
"machine",
"It",
"was",
"a",
"shame",
"how",
"he",
"carried",
"on",
"He",
"ruled",
"the",
"Russian",
"land",
"and",
"never",
"mind",
"the",
"Czar",
"But",
"the",
"kazachok",
"he",
"danced",
"really",
"wunderbar",
"In",
"all",
"affairs",
"of",
"state",
"he",
"was",
"the",
"man",
"to",
"please",
"But",
"he",
"was",
"real",
"great",
"when",
"he",
"had",
"a",
"girl",
"to",
"squeeze",
"For",
"the",
"queen",
"he",
"was",
"no",
"wheeler",
"dealer",
"Though",
"she'd",
"heard",
"the",
"things",
"he'd",
"done",
"She",
"believed",
"he",
"was",
"a",
"holy",
"healer",
"Who",
"would",
"heal",
"her",
"son",
"Ra-ra",
"Rasputin",
"Lover",
"of",
"the",
"Russian",
"queen",
"There",
"was",
"a",
"cat",
"that",
"really",
"was",
"gone",
"Ra-ra",
"Rasputin",
"Russia's",
"greatest",
"love",
"machine",
"It",
"was",
"a",
"shame",
"how",
"he",
"carried",
"on",
"But",
"when",
"his",
"drinking",
"and",
"lusting",
"And",
"his",
"hunger",
"for",
"power",
"Became",
"known",
"to",
"more",
"and",
"more",
"people",
"The",
"demands",
"to",
"do",
"something",
"About",
"this",
"outrageous",
"man",
"Became",
"louder",
"and",
"louder",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"Hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey,",
"hey",
"This",
"man's",
"just",
"got",
"to",
"go",
"declared",
"his",
"enemies",
"But",
"the",
"ladies",
"begged,",
"Don't",
"you",
"try",
"to",
"do",
"it,",
"please",
"No",
"doubt",
"this",
"Rasputin",
"had",
"lots",
"of",
"hidden",
"charms",
"Though",
"he",
"was",
"a",
"brute,",
"they",
"just",
"fell",
"into",
"his",
"arms",
"Then",
"one",
"night",
"some",
"men",
"of",
"higher",
"standing",
"Set",
"a",
"trap,",
"they're",
"not",
"to",
"blame",
"Come",
"to",
"visit",
"us",
"they",
"kept",
"demanding",
"And",
"he",
"really",
"came",
"Ra-ra",
"Rasputin",
"Lover",
"of",
"the",
"Russian",
"queen",
"They",
"put",
"some",
"poison",
"into",
"his",
"wine",
"Ra-ra",
"Rasputin",
"Russia's",
"greatest",
"love",
"machine",
"He",
"drank",
"it",
"all",
"and",
"said,",
"I",
"feel",
"fine",
"Ra-ra-",
"Rasputin",
"Lover",
"of",
"the",
"Russian",
"queen",
"They",
"didnt",
"quit,",
"they",
"wanted",
"his",
"head",
"Ra-ra-",
"Rasputin",
"Russia's",
"greatest",
"love",
"machine",
"And",
"so",
"they",
"shot",
"him",
"til",
"he",
"was",
"dead",
"Oh",
"those",
"Russians"
];