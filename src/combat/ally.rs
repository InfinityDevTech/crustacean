use crate::memory::ScreepsMemory;

use super::ally_syncing::sss::SSSAllySync;

pub struct Allies {
    pub sss: SSSAllySync,
}

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl Allies {
    pub fn new(memory: &mut ScreepsMemory) -> Self {
        Allies {
            sss: SSSAllySync::new(memory),
        }
    }

    pub fn sync(&mut self, memory: &mut ScreepsMemory) {
        self.sss.sync(memory);
    }
}