//allyModel.js
// alliance leader playerName. undefined to disable synchronization
const SYNC_PLAYER_BY_SHARD = {
	shard0: undefined,
	shard1: 'U-238',
	shard2: 'Winnduu',
	shard3: 'Shylo132',
};
// set to `true` if you are alliance leader
const SYNC_LEADER_BY_SHARD = {
	shard0: false,
	shard1: false,
	shard2: false,
	shard3: false,
};
// segment id for data synchronization
const SYNC_SEGMENT = 99;
// interval of synchronization in game ticks
const SYNC_INTERVAL = 100;

const SYNC_PLAYER = SYNC_PLAYER_BY_SHARD[Game.shard.name];
const SYNC_LEADER = SYNC_LEADER_BY_SHARD[Game.shard.name];

class AllyModel {
	
	constructor() {
		this.allies = {
			Shylo132: true,
			MerlinMan5: true,
			Starb: true,
			ThomasCui: false,
			Arigilos: true,
			PlainCucumber25: true,
			DollarAkshay: true,
			Pankpanther: true,
			"U-238": true,
			Winnduu: true,
			Salieri: true,
			ChuChuChu: true,
			Diesel13: true,
			Loop_Cat: true,
		}; // Object where keys are ally usernames and values are true
		
		if (SYNC_LEADER) {
			// save allies data to memory
			Memory.allies = this.allies;
			// save allies data to segment
			RawMemory.segments[SYNC_SEGMENT] = JSON.stringify({allies: this.allies});
			RawMemory.setPublicSegments([SYNC_SEGMENT]);
			RawMemory.setDefaultPublicSegment(SYNC_SEGMENT);
		} else {
			// load allies data from memory, if no memory set write default list of allies
			this.allies = Memory.allies || (Memory.allies = this.allies);
			this.nextSyncTime = Game.time;
		}
	}
	
	getSegment() {
		return SYNC_SEGMENT;
	}
	
	sync() {
		if (SYNC_LEADER || !SYNC_PLAYER || Game.time < this.nextSyncTime) {
			return;
		}
		
		const segment = RawMemory.foreignSegment;
		if (!segment || segment.username !== SYNC_PLAYER || segment.id !== SYNC_SEGMENT) {
			// set public segment for read and wait for next tick
			RawMemory.setActiveForeignSegment(SYNC_PLAYER, SYNC_SEGMENT);
			if (Game.time > this.nextSyncTime + 2) {
				console.log(`Error: foreignSegment ${SYNC_SEGMENT} of ${SYNC_PLAYER} was not loaded or could not be accessed`);
				this.nextSyncTime = Game.time + SYNC_INTERVAL;
			}
			return;
		}
		const data = JSON.parse(segment.data || '{}');
		if (typeof data !== 'object' || !data.allies) {
			console.log(`Error: alliance data is missing in ${SYNC_PLAYER} public segment ${SYNC_SEGMENT}`);
		} else {
			// alliance data loaded successfully
			this.allies = Memory.allies = data.allies;
		}
		this.nextSyncTime = Game.time + SYNC_INTERVAL;
	}
	
	// Method to check if a player is an ally
	isAlly(playerName) {
		return this.allies[playerName] === true;
	}
	
}
module.exports = new AllyModel();