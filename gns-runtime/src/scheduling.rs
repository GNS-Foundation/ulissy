//! Task scheduling - periodic and conditional execution
//! 
//! ULissy: `every 10.minutes { }`, `when condition { }`, `after 5.seconds { }`

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::duration::Duration;
use crate::error::RuntimeResult;


/// Handle to a scheduled task, allowing cancellation
#[derive(Clone)]
pub struct TaskHandle {
    cancel_flag: Arc<AtomicBool>,
    #[allow(dead_code)]
    name: String,
}

impl TaskHandle {
    fn new(name: impl Into<String>) -> Self {
        TaskHandle {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            name: name.into(),
        }
    }
    
    /// Cancel the scheduled task
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }
    
    /// Check if task has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }
}

/// Schedule a task to run periodically
/// 
/// ULissy:
/// ```ulissy
/// every 10.minutes {
///     collectBreadcrumb()
/// }
/// ```
/// 
/// # Arguments
/// * `interval` - How often to run the task
/// * `task` - The closure to execute
/// 
/// # Returns
/// A `TaskHandle` that can be used to cancel the task
#[cfg(feature = "tokio-runtime")]
pub fn schedule_every<F>(interval: Duration, mut task: F) -> RuntimeResult<TaskHandle>
where
    F: FnMut() -> RuntimeResult<()> + Send + 'static,
{
    let handle = TaskHandle::new("periodic_task");
    let cancel_flag = handle.cancel_flag.clone();
    
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(interval.to_std());
        
        loop {
            interval_timer.tick().await;
            
            if cancel_flag.load(Ordering::SeqCst) {
                tracing::debug!("Periodic task cancelled");
                break;
            }
            
            match task() {
                Ok(()) => {}
                Err(e) => {
                    tracing::error!("Periodic task error: {}", e);
                }
            }
        }
    });
    
    Ok(handle)
}

/// Schedule a task to run periodically with a condition
/// 
/// ULissy:
/// ```ulissy
/// every 10.minutes when battery > 20 {
///     collectBreadcrumb()
/// }
/// ```
#[cfg(feature = "tokio-runtime")]
pub fn schedule_every_when<F, C>(
    interval: Duration,
    mut condition: C,
    mut task: F,
) -> RuntimeResult<TaskHandle>
where
    F: FnMut() -> RuntimeResult<()> + Send + 'static,
    C: FnMut() -> bool + Send + 'static,
{
    let handle = TaskHandle::new("conditional_periodic_task");
    let cancel_flag = handle.cancel_flag.clone();
    
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(interval.to_std());
        
        loop {
            interval_timer.tick().await;
            
            if cancel_flag.load(Ordering::SeqCst) {
                tracing::debug!("Conditional periodic task cancelled");
                break;
            }
            
            // Only execute if condition is met
            if condition() {
                match task() {
                    Ok(()) => {}
                    Err(e) => {
                        tracing::error!("Conditional periodic task error: {}", e);
                    }
                }
            }
        }
    });
    
    Ok(handle)
}

/// Watch a condition and execute when it becomes true
/// 
/// ULissy:
/// ```ulissy
/// when me.trajectory.count >= 100 {
///     notifyHandleReady()
/// }
/// ```
/// 
/// The condition is checked periodically. When it becomes true,
/// the task executes once and the watcher stops.
#[cfg(feature = "tokio-runtime")]
pub fn watch_condition<F, C>(
    condition: C,
    task: F,
) -> RuntimeResult<TaskHandle>
where
    F: FnMut() -> RuntimeResult<()> + Send + 'static,
    C: FnMut() -> bool + Send + 'static,
{
    watch_condition_with_interval(
        Duration::from_secs(1), // Check every second by default
        condition,
        task,
    )
}

/// Watch a condition with custom check interval
#[cfg(feature = "tokio-runtime")]
pub fn watch_condition_with_interval<F, C>(
    check_interval: Duration,
    mut condition: C,
    mut task: F,
) -> RuntimeResult<TaskHandle>
where
    F: FnMut() -> RuntimeResult<()> + Send + 'static,
    C: FnMut() -> bool + Send + 'static,
{
    let handle = TaskHandle::new("condition_watcher");
    let cancel_flag = handle.cancel_flag.clone();
    
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(check_interval.to_std());
        
        loop {
            interval_timer.tick().await;
            
            if cancel_flag.load(Ordering::SeqCst) {
                tracing::debug!("Condition watcher cancelled");
                break;
            }
            
            // Check condition
            if condition() {
                tracing::debug!("Condition met, executing task");
                match task() {
                    Ok(()) => {}
                    Err(e) => {
                        tracing::error!("Condition task error: {}", e);
                    }
                }
                // Task executed once, stop watching
                break;
            }
        }
    });
    
    Ok(handle)
}

/// Watch a condition continuously (executes every time condition is true)
#[cfg(feature = "tokio-runtime")]
pub fn watch_condition_continuous<F, C>(
    check_interval: Duration,
    mut condition: C,
    mut task: F,
) -> RuntimeResult<TaskHandle>
where
    F: FnMut() -> RuntimeResult<()> + Send + 'static,
    C: FnMut() -> bool + Send + 'static,
{
    let handle = TaskHandle::new("continuous_condition_watcher");
    let cancel_flag = handle.cancel_flag.clone();
    
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(check_interval.to_std());
        let mut was_true = false;
        
        loop {
            interval_timer.tick().await;
            
            if cancel_flag.load(Ordering::SeqCst) {
                break;
            }
            
            let is_true = condition();
            
            // Execute on rising edge (false -> true transition)
            if is_true && !was_true {
                match task() {
                    Ok(()) => {}
                    Err(e) => {
                        tracing::error!("Continuous condition task error: {}", e);
                    }
                }
            }
            
            was_true = is_true;
        }
    });
    
    Ok(handle)
}

/// Execute a task after a delay
/// 
/// ULissy:
/// ```ulissy
/// after 5.seconds {
///     dismissNotification()
/// }
/// ```
#[cfg(feature = "tokio-runtime")]
pub fn delay<F>(duration: Duration, task: F) -> RuntimeResult<TaskHandle>
where
    F: FnOnce() -> RuntimeResult<()> + Send + 'static,
{
    let handle = TaskHandle::new("delayed_task");
    let cancel_flag = handle.cancel_flag.clone();
    
    tokio::spawn(async move {
        tokio::time::sleep(duration.to_std()).await;
        
        if !cancel_flag.load(Ordering::SeqCst) {
            match task() {
                Ok(()) => {}
                Err(e) => {
                    tracing::error!("Delayed task error: {}", e);
                }
            }
        }
    });
    
    Ok(handle)
}

/// Execute a task after a delay (alias for `delay`)
#[cfg(feature = "tokio-runtime")]
pub fn after<F>(duration: Duration, task: F) -> RuntimeResult<TaskHandle>
where
    F: FnOnce() -> RuntimeResult<()> + Send + 'static,
{
    delay(duration, task)
}

// Non-tokio fallback implementations (synchronous)

#[cfg(not(feature = "tokio-runtime"))]
pub fn schedule_every<F>(_interval: Duration, _task: F) -> RuntimeResult<TaskHandle>
where
    F: FnMut() -> RuntimeResult<()> + Send + 'static,
{
    Err(RuntimeError::SchedulingError(
        "Async runtime not enabled. Enable 'tokio-runtime' feature.".into()
    ))
}

#[cfg(not(feature = "tokio-runtime"))]
pub fn watch_condition<F, C>(_condition: C, _task: F) -> RuntimeResult<TaskHandle>
where
    F: FnMut() -> RuntimeResult<()> + Send + 'static,
    C: FnMut() -> bool + Send + 'static,
{
    Err(RuntimeError::SchedulingError(
        "Async runtime not enabled. Enable 'tokio-runtime' feature.".into()
    ))
}

#[cfg(not(feature = "tokio-runtime"))]
pub fn delay<F>(_duration: Duration, _task: F) -> RuntimeResult<TaskHandle>
where
    F: FnOnce() -> RuntimeResult<()> + Send + 'static,
{
    Err(RuntimeError::SchedulingError(
        "Async runtime not enabled. Enable 'tokio-runtime' feature.".into()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;
    
    #[cfg(feature = "tokio-runtime")]
    #[tokio::test]
    async fn test_task_handle_cancellation() {
        let handle = TaskHandle::new("test");
        assert!(!handle.is_cancelled());
        
        handle.cancel();
        assert!(handle.is_cancelled());
    }
    
    #[cfg(feature = "tokio-runtime")]
    #[tokio::test]
    async fn test_delay() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        delay(Duration::from_millis(50), move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }).unwrap();
        
        // Wait for delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
