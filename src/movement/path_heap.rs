

use crate::constants::PATHFINDER_MAX_ROOMS;

pub struct PathHeap {
    priorities: Box<[u32; (2500 * PATHFINDER_MAX_ROOMS) as usize]>,
    heap: Box<[u32; ((2500 * PATHFINDER_MAX_ROOMS) / 8) as usize]>,
    size: usize,
}

impl PathHeap {
    pub fn new() -> PathHeap {
        PathHeap {
            priorities: Box::new([0; (2500 * PATHFINDER_MAX_ROOMS) as usize]),
            heap: Box::new([0; (2500 * (PATHFINDER_MAX_ROOMS / 8)) as usize]),
            size: 0,
        }
    }

    pub fn empty(&self) -> bool {
        self.size == 0
    }

    pub fn priority(&self, index: usize) -> u32 {
        self.priorities[index]
    }

    pub fn pop(&mut self) -> (u32, u32) {
        let ret = (self.heap[1], self.priorities[self.heap[1] as usize]);
        self.heap[1] = self.heap[self.size];
        self.size -= 1;
        let mut vv = 1;
        loop {
            let uu = vv;

            if (uu << 1) < self.size {
                if self.priorities[self.heap[uu] as usize] >= self.priorities[self.heap[uu << 1] as usize] {
                    vv = uu << 1;
                }
                if self.priorities[self.heap[vv] as usize] >= self.priorities[self.heap[(uu << 1) + 1] as usize] {
                    vv = (uu << 1) + 1;
                }
            } else if uu << 1 <= self.size && self.priorities[self.heap[uu] as usize] >= self.priorities[self.heap[uu << 1] as usize] {
                vv = uu << 1;
            }

            if uu != vv {
                self.heap.swap(uu, vv);
            } else {
                break;
            }
        }

        ret
    }

    pub fn insert(&mut self, index: u32, priority: u32) {
        self.priorities[index as usize] = priority;
        self.size += 1;
        self.heap[self.size] = index;
        self.bubble_up(self.size);
    }

    pub fn update(&mut self, index: u32, priority: u32) {
        for i in 1..=self.size {
            if self.heap[i] == index {
                self.priorities[i] = priority;
                self.bubble_up(i);
                break;
            }
        }
    }

    pub fn bubble_up(&mut self, mut index: usize) {
        while index != 1 {
            if self.priorities[self.heap[index] as usize] <= self.priorities[self.heap[index >> 1] as usize] {
                self.heap.swap(index, index >> 1);
                index >>= 1;
            } else {
                break;
            }
        }
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }
}

// 0 - unknown
// 1 - open
// 2 - closed
pub struct OpenClose {
    open_close: [u8; 2500 * PATHFINDER_MAX_ROOMS as usize],
    marker: u8,
}

impl OpenClose {
    pub fn new() -> OpenClose {
        OpenClose {
            open_close: [0; 2500 * PATHFINDER_MAX_ROOMS as usize],
            marker: 1,
        }
    }

    pub fn clear(&mut self) {
        if u8::MAX - 2 <= self.marker {
            self.open_close.fill(0);
            self.marker = 1;
        } else {
            self.marker += 2;
        }
    }

    pub fn is_open(&self, index: u32) -> bool {
        self.open_close[index as usize] == self.marker
    }

    pub fn is_closed(&self, index: u32) -> bool {
        self.open_close[index as usize] == self.marker + 1
    }

    pub fn open(&mut self, index: u32) {
        self.open_close[index as usize] = self.marker;
    }

    pub fn close(&mut self, index: u32) {
        self.open_close[index as usize] = self.marker + 1;
    }
}