//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: Vec<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: Vec::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(task);
    }
    /// take the process whose stride is smallest out of the ready queue and update its stride
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let min_stride_tcb = 
        self.ready_queue
        .iter()
        .min_by_key(|tcb|tcb.inner_exclusive_access().get_stride())
        .cloned();

        if let Some(tcb) = min_stride_tcb {
            let tcb_clone = tcb.clone();
            let mut tcb_inner = tcb.inner_exclusive_access();
            tcb_inner.update_stride();
            let pid = tcb.getpid();
            self.ready_queue.retain(|x| x.getpid() != pid);
            Some(tcb_clone)
        } else {
            None
        }
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
