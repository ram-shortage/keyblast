/// Async macro execution infrastructure.
///
/// Provides non-blocking macro execution with cancellation support.
/// Worker thread handles timing/delays; commands flow to main thread via channel.
///
/// # Architecture
///
/// On macOS, keystroke injection must occur on the main thread (TIS/TSM requirement).
/// This module spawns a worker thread for timing coordination while sending individual
/// keystroke commands back to the main thread for execution.
///
/// ```text
/// Worker Thread                    Main Thread
/// +--------------+                +--------------+
/// | for segment  |  -- Inject --> | receive cmd  |
/// | in segments  |                | execute key  |
/// | sleep delay  |                | check stop   |
/// +--------------+                +--------------+
/// ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::injection::MacroSegment;

/// Command sent from worker thread to main thread.
#[derive(Debug)]
pub enum ExecutionCommand {
    /// Execute a single macro segment on main thread.
    Inject(MacroSegment),
    /// Execution completed successfully.
    Complete,
    /// Execution was cancelled by user.
    Cancelled,
}

/// Handle for controlling a running macro execution.
///
/// Provides methods to request cancellation and check execution status.
/// The handle owns the worker thread and should be joined on app exit.
pub struct ExecutionHandle {
    /// Set to true to request cancellation.
    stop_flag: Arc<AtomicBool>,
    /// Thread handle for cleanup.
    thread: Option<JoinHandle<()>>,
}

impl ExecutionHandle {
    /// Request the execution to stop.
    ///
    /// This sets a flag that the worker thread checks between segments.
    /// Cancellation is not immediate but will occur before the next segment.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    /// Check if the worker thread is still running.
    pub fn is_running(&self) -> bool {
        self.thread.as_ref().map_or(false, |t| !t.is_finished())
    }

    /// Wait for the worker thread to complete.
    ///
    /// Call this on app exit to ensure clean shutdown.
    pub fn join(mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Start async execution of a macro.
///
/// Spawns a worker thread that iterates through segments, sending each to the main
/// thread for execution. The worker handles timing (delays between segments) while
/// the main thread handles the actual keystroke injection.
///
/// # Arguments
///
/// * `segments` - The macro segments to execute
/// * `delay_ms` - Delay between segments in milliseconds
///
/// # Returns
///
/// A tuple of:
/// * `Receiver<ExecutionCommand>` - Process these commands in the main event loop
/// * `ExecutionHandle` - Use to request cancellation or wait for completion
///
/// # Example
///
/// ```ignore
/// let segments = parse_macro_sequence("Hello{Enter}World");
/// let (rx, handle) = start_execution(segments, 50);
///
/// // In event loop:
/// while let Ok(cmd) = rx.try_recv() {
///     match cmd {
///         ExecutionCommand::Inject(segment) => injector.execute_single_segment(&segment),
///         ExecutionCommand::Complete => println!("Done!"),
///         ExecutionCommand::Cancelled => println!("Stopped"),
///     }
/// }
///
/// // To cancel:
/// handle.stop();
/// ```
pub fn start_execution(
    segments: Vec<MacroSegment>,
    delay_ms: u64,
) -> (Receiver<ExecutionCommand>, ExecutionHandle) {
    let (tx, rx) = unbounded();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    let thread = std::thread::spawn(move || {
        execution_worker(segments, delay_ms, stop_flag_clone, tx);
    });

    let handle = ExecutionHandle {
        stop_flag,
        thread: Some(thread),
    };

    (rx, handle)
}

/// Worker thread function.
///
/// Iterates through segments, checking the stop flag before each.
/// Sends segments to main thread via channel.
///
/// Key timing behaviors:
/// - {Delay N} segments: worker sleeps (doesn't send to main thread)
/// - Text segments with delay_ms > 0: split into per-character injections
/// - All other segments: sent to main thread, worker sleeps delay_ms after
fn execution_worker(
    segments: Vec<MacroSegment>,
    delay_ms: u64,
    stop_flag: Arc<AtomicBool>,
    tx: Sender<ExecutionCommand>,
) {
    // Expand segments: Text with delay_ms > 0 becomes per-character
    let expanded: Vec<MacroSegment> = if delay_ms > 0 {
        segments.into_iter().flat_map(|seg| {
            match seg {
                MacroSegment::Text(text) => {
                    // Split text into individual characters for per-char delay
                    text.chars()
                        .map(|c| MacroSegment::Text(c.to_string()))
                        .collect::<Vec<_>>()
                }
                other => vec![other],
            }
        }).collect()
    } else {
        segments
    };

    let segment_count = expanded.len();

    for (i, segment) in expanded.into_iter().enumerate() {
        // Check for cancellation before each segment
        if stop_flag.load(Ordering::Relaxed) {
            let _ = tx.send(ExecutionCommand::Cancelled);
            return;
        }

        // Handle Delay segments in worker thread (don't block main thread)
        if let MacroSegment::Delay(ms) = segment {
            if !cancellable_sleep(ms, &stop_flag) {
                let _ = tx.send(ExecutionCommand::Cancelled);
                return;
            }
            continue; // Don't send Delay to main thread
        }

        // Send segment to main thread for execution
        if tx.send(ExecutionCommand::Inject(segment)).is_err() {
            // Receiver dropped, exit gracefully
            return;
        }

        // Wait between segments if delay specified (not after last segment)
        if delay_ms > 0 && i < segment_count.saturating_sub(1) {
            if !cancellable_sleep(delay_ms, &stop_flag) {
                let _ = tx.send(ExecutionCommand::Cancelled);
                return;
            }
        }
    }

    let _ = tx.send(ExecutionCommand::Complete);
}

/// Sleep for the specified duration, checking the stop flag periodically.
/// Returns true if sleep completed, false if cancelled.
fn cancellable_sleep(ms: u64, stop_flag: &Arc<AtomicBool>) -> bool {
    let check_interval = Duration::from_millis(50.min(ms));
    let total_delay = Duration::from_millis(ms);
    let start = Instant::now();

    while start.elapsed() < total_delay {
        if stop_flag.load(Ordering::Relaxed) {
            return false;
        }
        std::thread::sleep(check_interval);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use enigo::Key;

    #[test]
    fn test_execution_command_debug() {
        // Ensure ExecutionCommand derives Debug correctly
        let cmd = ExecutionCommand::Complete;
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Complete"));
    }

    #[test]
    fn test_start_execution_returns_receiver_and_handle() {
        let segments = vec![MacroSegment::Text("test".to_string())];
        let (rx, handle) = start_execution(segments, 0);

        // Should receive the segment and completion
        // Give thread time to run
        std::thread::sleep(Duration::from_millis(50));

        let mut received_inject = false;
        let mut received_complete = false;

        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                ExecutionCommand::Inject(_) => received_inject = true,
                ExecutionCommand::Complete => received_complete = true,
                ExecutionCommand::Cancelled => {}
            }
        }

        assert!(received_inject, "Should receive Inject command");
        assert!(received_complete, "Should receive Complete command");

        // Clean up
        handle.join();
    }

    #[test]
    fn test_execution_stop_flag() {
        // Create segments with delay to allow time for cancellation
        let segments = vec![
            MacroSegment::Text("a".to_string()),
            MacroSegment::Text("b".to_string()),
            MacroSegment::Text("c".to_string()),
        ];

        let (rx, handle) = start_execution(segments, 200); // 200ms delay

        // Wait a bit then request stop
        std::thread::sleep(Duration::from_millis(50));
        handle.stop();

        // Wait for worker to finish
        std::thread::sleep(Duration::from_millis(100));

        // Should have received at least one inject and a cancelled
        let mut received_cancelled = false;
        while let Ok(cmd) = rx.try_recv() {
            if matches!(cmd, ExecutionCommand::Cancelled) {
                received_cancelled = true;
            }
        }

        assert!(received_cancelled, "Should receive Cancelled after stop");
        handle.join();
    }

    #[test]
    fn test_execution_handle_is_running() {
        let segments = vec![MacroSegment::Text("test".to_string())];
        let (_rx, handle) = start_execution(segments, 0);

        // Thread should finish quickly with no delay
        std::thread::sleep(Duration::from_millis(50));
        assert!(!handle.is_running(), "Thread should have finished");

        handle.join();
    }

    #[test]
    fn test_execution_multiple_segments() {
        let segments = vec![
            MacroSegment::Text("Hello".to_string()),
            MacroSegment::SpecialKey(Key::Return),
            MacroSegment::Text("World".to_string()),
        ];

        let (rx, handle) = start_execution(segments, 0);

        std::thread::sleep(Duration::from_millis(50));

        let mut inject_count = 0;
        let mut completed = false;

        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                ExecutionCommand::Inject(_) => inject_count += 1,
                ExecutionCommand::Complete => completed = true,
                ExecutionCommand::Cancelled => {}
            }
        }

        assert_eq!(inject_count, 3, "Should receive 3 Inject commands");
        assert!(completed, "Should receive Complete command");

        handle.join();
    }
}
