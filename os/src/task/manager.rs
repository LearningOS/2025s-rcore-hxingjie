//!Implementation of [`TaskManager`]

use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        if self.ready_queue.is_empty() {
            None
        } else if self.ready_queue.len() == 1 {
            self.ready_queue.pop_front()
        } else {
            let mut min_stride = self.ready_queue[0].inner_exclusive_access().stride;
            let mut min_idx = 0;
            for (idx, task) in self.ready_queue.iter().enumerate() {
                if task.inner_exclusive_access().stride < min_stride {
                    min_stride = task.inner_exclusive_access().stride;
                    min_idx = idx;
                }
            }
            self.ready_queue.swap(0, min_idx);
            let task = self.ready_queue.pop_front().unwrap();
            let mut inner = task.inner_exclusive_access();
            inner.stride += 1_000_000 / inner.priority;
            drop(inner);
            
            Some(task)
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
