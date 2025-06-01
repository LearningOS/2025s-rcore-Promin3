//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

use alloc::vec::Vec;
use alloc::vec;
pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;

/// Deadlock detection module
pub struct DeadlockDetect{
    /// Available[j] = k，表示第 j 类资源的可用数量为 k
    pub available: Vec<usize>,
    /// Allocation[i,j] = g，则表示线程 i 当前己分得第 j 类资源的数量为 g
    pub allocation: Vec<Vec<usize>>,
    /// Need[i,j] = d，则表示线程 i 还需要第 j 类资源的数量为 d 
    pub need: Vec<Vec<usize>>,
}

impl DeadlockDetect {
    /// Create a new deadlock detector
    pub fn new()-> Self {
        trace!("kernel: DeadlockDetect::new");
        Self {
            available: Vec::new(),
            allocation: Vec::new(),
            need: Vec::new(),
        }
    }
    
    /// add available resource
    pub fn add_available(&mut self, count: usize) {
        self.available.push(count);
        for i in 0..self.allocation.len() {
            self.allocation[i].push(0);
            self.need[i].push(0);
        }
    }

    /// set available resource 
    pub fn set_available(&mut self, index: usize, count: usize) {
        self.available[index] = count;
        for i in 0..self.allocation.len() {
            self.allocation[i][index] = 0;
            self.need[i][index] = 0;
        }
    }

    /// DeadlockDetect change when a thread was created
    pub fn add_thread(&mut self, tid: usize){
        let col = vec![0; self.available.len()];
        while self.allocation.len() < tid + 1 {
            self.allocation.push(col.clone());
            self.need.push(col.clone());
        }
    }

    /// DeadlockDetect change when a thread was destroyed
    pub fn release_thread(&mut self, tid: usize) {
        for i in 0..self.available.len() {
            self.available[i] += self.allocation[tid][i];
            self.allocation[tid][i] = 0;
            self.need[tid][i] = 0;
        }
    }

    /// algorithm to detect deadlock
    pub fn check_deadlock(&self) -> bool {
        trace!("kernel: DeadlockDetect::check_deadlock");
        let mut work = self.available.clone();
        let mut finish = vec![false; self.allocation.len()];
        while let Some(id) = self.find_thread(&mut work, &mut finish) {
            for j in 0..self.available.len() {
                work[j] += self.allocation[id][j]; // add allocation to work
            }
            finish[id] = true; // mark this thread as finished
        }
        finish.iter().any(|&f| !f) 
    }

    /// find a thread that can finish
    fn find_thread(&self, work: &mut Vec<usize>, finish: &mut Vec<bool>) -> Option<usize> {
        for i in 0..self.allocation.len() {
            if finish[i] {
                continue; // this thread has finished
            }
            let mut can_finish = true;
            for j in 0..self.available.len() {
                if self.need[i][j] > work[j] {
                    can_finish = false; // this thread cannot finish
                    break;
                }
            }
            if can_finish{
                return Some(i); // this thread can finish
            }
        }   
        None
    }

}