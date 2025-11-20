//! Parallel execution engine for scalable operations
//!
//! Provides concurrent task execution with configurable parallelism.

use crate::{Error, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use tokio::sync::Semaphore;
use tracing::{debug, error, info};

/// Task to be executed
pub struct Task {
    pub id: usize,
    pub name: String,
    pub dependencies: Vec<usize>,
    pub work: Box<dyn FnOnce() -> Result<TaskOutput> + Send>,
}

/// Output from a task
#[derive(Clone)]
pub struct TaskOutput {
    pub id: usize,
    pub data: Vec<u8>,
}

/// Result of task execution
#[derive(Clone)]
pub struct TaskResult {
    pub id: usize,
    pub name: String,
    pub success: bool,
    pub output: Option<TaskOutput>,
    pub error: Option<String>,
    pub duration: std::time::Duration,
}

/// Parallel executor
pub struct ParallelExecutor {
    parallelism: usize,
    semaphore: Arc<Semaphore>,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(parallelism: usize) -> Self {
        let parallelism = parallelism.max(1);
        Self {
            parallelism,
            semaphore: Arc::new(Semaphore::new(parallelism)),
        }
    }

    /// Execute tasks with dependency ordering
    pub async fn execute(&self, tasks: Vec<Task>) -> Result<Vec<TaskResult>> {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        info!(
            "Executing {} tasks with parallelism {}",
            tasks.len(),
            self.parallelism
        );

        let total_tasks = tasks.len();
        let completed = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));

        // Build dependency graph
        let mut pending: HashMap<usize, Task> = HashMap::new();
        let mut dep_count: HashMap<usize, usize> = HashMap::new();
        let mut dependents: HashMap<usize, Vec<usize>> = HashMap::new();

        for task in tasks {
            let id = task.id;
            dep_count.insert(id, task.dependencies.len());

            for &dep in &task.dependencies {
                dependents.entry(dep).or_default().push(id);
            }

            pending.insert(id, task);
        }

        // Find initially ready tasks (no dependencies)
        let mut ready: Vec<usize> = dep_count
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(&id, _)| id)
            .collect();

        let pending = Arc::new(Mutex::new(pending));
        let dep_count = Arc::new(Mutex::new(dep_count));
        let dependents = Arc::new(Mutex::new(dependents));
        let ready = Arc::new(Mutex::new(ready));
        let results = Arc::new(Mutex::new(Vec::new()));

        // Process tasks
        loop {
            // Check for cancellation
            if cancelled.load(Ordering::Relaxed) {
                break;
            }

            // Get ready tasks
            let task_ids: Vec<usize> = {
                let mut ready = ready.lock();
                ready.drain(..).collect()
            };

            if task_ids.is_empty() {
                // Check if we're done
                let completed_count = completed.load(Ordering::Relaxed);
                if completed_count >= total_tasks {
                    break;
                }

                // Wait for tasks to complete
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                continue;
            }

            // Spawn tasks
            let mut handles = Vec::new();

            for task_id in task_ids {
                let task = {
                    let mut pending = pending.lock();
                    pending.remove(&task_id)
                };

                let Some(task) = task else {
                    continue;
                };

                let semaphore = self.semaphore.clone();
                let completed = completed.clone();
                let cancelled = cancelled.clone();
                let dep_count = dep_count.clone();
                let dependents = dependents.clone();
                let ready_arc = ready.clone();
                let results = results.clone();

                let handle = tokio::spawn(async move {
                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.unwrap();

                    if cancelled.load(Ordering::Relaxed) {
                        return;
                    }

                    let start = std::time::Instant::now();
                    let task_name = task.name.clone();
                    let task_id = task.id;

                    debug!("Starting task {}: {}", task_id, task_name);

                    // Execute task
                    let result = match (task.work)() {
                        Ok(output) => TaskResult {
                            id: task_id,
                            name: task_name,
                            success: true,
                            output: Some(output),
                            error: None,
                            duration: start.elapsed(),
                        },
                        Err(e) => {
                            error!("Task {} failed: {}", task_id, e);
                            TaskResult {
                                id: task_id,
                                name: task_name,
                                success: false,
                                output: None,
                                error: Some(e.to_string()),
                                duration: start.elapsed(),
                            }
                        }
                    };

                    // Store result
                    results.lock().push(result.clone());

                    // Update completed count
                    completed.fetch_add(1, Ordering::Relaxed);

                    // If failed, cancel remaining tasks
                    if !result.success {
                        cancelled.store(true, Ordering::Relaxed);
                        return;
                    }

                    // Update dependents
                    let deps = dependents.lock();
                    if let Some(dependent_ids) = deps.get(&task_id) {
                        let mut dep_count = dep_count.lock();
                        let mut ready = ready_arc.lock();

                        for &dep_id in dependent_ids {
                            if let Some(count) = dep_count.get_mut(&dep_id) {
                                *count = count.saturating_sub(1);
                                if *count == 0 {
                                    ready.push(dep_id);
                                }
                            }
                        }
                    }
                });

                handles.push(handle);
            }

            // Wait for this batch
            for handle in handles {
                let _ = handle.await;
            }
        }

        let results = Arc::try_unwrap(results)
            .map_err(|_| Error::Other("Failed to get results".to_string()))?
            .into_inner();

        // Check for failures
        let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();
        if !failed.is_empty() {
            let names: Vec<_> = failed.iter().map(|r| r.name.clone()).collect();
            return Err(Error::TransactionFailed(format!(
                "Tasks failed: {}",
                names.join(", ")
            )));
        }

        Ok(results)
    }

    /// Execute a simple function in parallel
    pub async fn map<T, R, F>(&self, items: Vec<T>, f: F) -> Result<Vec<R>>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> Result<R> + Send + Sync + Clone + 'static,
    {
        let results: Arc<Mutex<Vec<(usize, Result<R>)>>> =
            Arc::new(Mutex::new(Vec::with_capacity(items.len())));
        let mut handles = Vec::new();

        for (idx, item) in items.into_iter().enumerate() {
            let semaphore = self.semaphore.clone();
            let results = results.clone();
            let f = f.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = f(item);
                (idx, result)
            });

            handles.push(handle);
        }

        let mut indexed_results = Vec::new();
        for handle in handles {
            let (idx, result) = handle.await.map_err(|e| Error::Other(e.to_string()))?;
            indexed_results.push((idx, result));
        }

        // Sort by index
        indexed_results.sort_by_key(|(idx, _)| *idx);

        // Extract results
        indexed_results.into_iter().map(|(_, r)| r).collect()
    }

    /// Get current parallelism level
    pub fn parallelism(&self) -> usize {
        self.parallelism
    }
}

/// Thread pool executor for CPU-bound tasks
pub struct ThreadPoolExecutor {
    sender: Sender<Box<dyn FnOnce() + Send>>,
    #[allow(dead_code)]
    workers: Vec<thread::JoinHandle<()>>,
}

impl ThreadPoolExecutor {
    /// Create a new thread pool
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver): (
            Sender<Box<dyn FnOnce() + Send>>,
            Receiver<Box<dyn FnOnce() + Send>>,
        ) = bounded(num_threads * 2);

        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let receiver = receiver.clone();
            let handle = thread::spawn(move || {
                loop {
                    let task = {
                        let receiver = receiver.lock();
                        receiver.recv()
                    };

                    match task {
                        Ok(task) => task(),
                        Err(_) => break, // Channel closed
                    }
                }
            });
            workers.push(handle);
        }

        Self { sender, workers }
    }

    /// Submit a task to the pool
    pub fn submit<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .send(Box::new(f))
            .map_err(|_| Error::Other("Thread pool closed".to_string()))
    }
}

impl Drop for ThreadPoolExecutor {
    fn drop(&mut self) {
        // Drop sender to signal workers to exit
        drop(self.sender.clone());
    }
}
