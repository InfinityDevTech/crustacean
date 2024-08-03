use screeps::{ObjectId, Source};

#[derive(Debug, Clone, Default)]
pub struct HeapRoom {
    pub sources: Vec<ObjectId<Source>>,
}