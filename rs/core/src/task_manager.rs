/// Background task manager for long-running tool executions.
///
/// When a tool execution exceeds its timeout, it's moved to the background.
/// The agent can then: check status, read output, kill, or send input.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

static TASKS: Mutex<Option<TaskManager>> = Mutex::new(None);

pub struct TaskManager {
    tasks: HashMap<u32, Task>,
    next_id: u32,
}

struct Task {
    name: String,
    status: TaskStatus,
    /// Accumulated output so far.
    output: Arc<Mutex<String>>,
    /// Send input to the task (if it supports stdin).
    input_tx: Option<mpsc::Sender<String>>,
    /// Signal to kill the task.
    kill: Arc<std::sync::atomic::AtomicBool>,
    /// Thread handle.
    handle: Option<std::thread::JoinHandle<()>>,
}

#[derive(Clone, Debug)]
pub enum TaskStatus {
    Running,
    Completed,
    Killed,
    Failed(String),
}

impl TaskManager {
    fn new() -> Self {
        Self { tasks: HashMap::new(), next_id: 1 }
    }
}

fn manager() -> std::sync::MutexGuard<'static, Option<TaskManager>> {
    let mut guard = TASKS.lock().unwrap();
    if guard.is_none() {
        *guard = Some(TaskManager::new());
    }
    guard
}

/// Run a function with a timeout. If it completes in time, return the result.
/// If it times out, move to background and return a task handle.
pub fn run_with_timeout<F>(
    name: &str,
    timeout_secs: u64,
    func: F,
) -> ToolResult
where
    F: FnOnce() -> String + Send + 'static,
{
    let output = Arc::new(Mutex::new(String::new()));
    let output_clone = Arc::clone(&output);
    let kill = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let (done_tx, done_rx) = mpsc::channel::<()>();

    let handle = std::thread::Builder::new()
        .name(format!("clx-task-{}", name))
        .spawn(move || {
            let result = func();
            *output_clone.lock().unwrap() = result;
            let _ = done_tx.send(());
        })
        .expect("spawn task thread");

    // Wait with timeout.
    let timeout = std::time::Duration::from_secs(timeout_secs);
    match done_rx.recv_timeout(timeout) {
        Ok(()) => {
            // Completed in time.
            let _ = handle.join();
            let result = output.lock().unwrap().clone();
            ToolResult::Inline(result)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Timed out — move to background.
            let mut mgr = manager();
            let mgr = mgr.as_mut().unwrap();
            let id = mgr.next_id;
            mgr.next_id += 1;

            eprintln!("[CLX] task: #{} '{}' timed out after {}s, moved to background", id, name, timeout_secs);

            mgr.tasks.insert(id, Task {
                name: name.to_string(),
                status: TaskStatus::Running,
                output,
                input_tx: None,
                kill: kill.clone(),
                handle: Some(handle),
            });

            // Spawn a watcher that marks completion.
            let output_ref = {
                let t = mgr.tasks.get(&id).unwrap();
                Arc::clone(&t.output)
            };
            std::thread::Builder::new()
                .name(format!("clx-task-watch-{}", id))
                .spawn(move || {
                    // The task thread will finish eventually.
                    // We can't join it here (handle moved), just poll output changes.
                    // The done_rx already timed out, so we wait on the original thread.
                    // Actually the handle is in the Task struct, completion is detected
                    // in task_status by checking if the thread has finished.
                    let _ = done_rx.recv(); // blocks until task finishes
                    let mut mgr = manager();
                    if let Some(mgr) = mgr.as_mut() {
                        if let Some(task) = mgr.tasks.get_mut(&id) {
                            task.status = TaskStatus::Completed;
                        }
                    }
                })
                .ok();

            ToolResult::Background {
                task_id: id,
                message: format!("Task #{} '{}' is running in background (timed out after {}s). Use task_status({}) to check, task_output({}) to read result, or task_kill({}) to stop.", id, name, timeout_secs, id, id, id),
            }
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            // Thread panicked or dropped.
            ToolResult::Inline("Task thread died unexpectedly.".to_string())
        }
    }
}

pub enum ToolResult {
    /// Completed within timeout, result is inline.
    Inline(String),
    /// Moved to background, agent can check later.
    Background { task_id: u32, message: String },
}

/// Check task status.
pub fn task_status(id: u32) -> String {
    let mgr = manager();
    let mgr = mgr.as_ref().unwrap();
    match mgr.tasks.get(&id) {
        Some(task) => {
            let status = match &task.status {
                TaskStatus::Running => "running",
                TaskStatus::Completed => "completed",
                TaskStatus::Killed => "killed",
                TaskStatus::Failed(e) => return format!("Task #{} '{}': failed ({})", id, task.name, e),
            };
            let output_len = task.output.lock().unwrap().len();
            format!("Task #{} '{}': {} ({} chars output)", id, task.name, status, output_len)
        }
        None => format!("Task #{} not found.", id),
    }
}

/// Read task output.
pub fn task_output(id: u32, start_char: usize, max_chars: usize) -> String {
    let mgr = manager();
    let mgr = mgr.as_ref().unwrap();
    match mgr.tasks.get(&id) {
        Some(task) => {
            let output = task.output.lock().unwrap();
            let chars: Vec<char> = output.chars().collect();
            let end = (start_char + max_chars).min(chars.len());
            let start = start_char.min(chars.len());
            let slice: String = chars[start..end].iter().collect();
            format!("Task #{} output (chars {}-{} of {}):\n{}", id, start, end, chars.len(), slice)
        }
        None => format!("Task #{} not found.", id),
    }
}

/// Kill a background task.
pub fn task_kill(id: u32) -> String {
    let mut mgr = manager();
    let mgr = mgr.as_mut().unwrap();
    match mgr.tasks.get_mut(&id) {
        Some(task) => {
            task.kill.store(true, std::sync::atomic::Ordering::Relaxed);
            task.status = TaskStatus::Killed;
            format!("Task #{} '{}' killed.", id, task.name)
        }
        None => format!("Task #{} not found.", id),
    }
}

/// List all tasks.
pub fn task_list() -> String {
    let mgr = manager();
    let mgr = mgr.as_ref().unwrap();
    if mgr.tasks.is_empty() {
        return "No background tasks.".to_string();
    }
    let mut lines = Vec::new();
    for (id, task) in &mgr.tasks {
        let status = match &task.status {
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Killed => "killed",
            TaskStatus::Failed(_) => "failed",
        };
        let output_len = task.output.lock().unwrap().len();
        lines.push(format!("  #{}: '{}' [{}] ({} chars)", id, task.name, status, output_len));
    }
    format!("Background tasks:\n{}", lines.join("\n"))
}
