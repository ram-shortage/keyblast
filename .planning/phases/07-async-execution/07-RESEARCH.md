# Phase 7: Async Execution - Research

**Researched:** 2026-01-17
**Domain:** Rust threading, cross-thread communication, cancellation patterns, keystroke injection thread safety
**Confidence:** HIGH

## Summary

Phase 7 transforms KeyBlast's macro execution from blocking synchronous calls to non-blocking background execution with user-cancellable operations. The research reveals a critical platform constraint: on macOS, keystroke injection via enigo **must occur on the main thread** due to TIS/TSM (Text Input Source/Text Services Manager) requirements. This fundamentally shapes the architecture.

The recommended approach is a **"chunked execution with yield" pattern**: the macro executes on the main thread but yields control back to the event loop between keystroke segments, allowing the UI to remain responsive. For truly long-running macros with delays, a hybrid approach uses a background thread for timing/coordination while sending individual keystroke commands back to the main thread for execution.

The project already has `crossbeam-channel` as a dependency (though unused), which is the ideal choice for cross-thread communication. For cancellation, `Arc<AtomicBool>` provides a simple, dependency-free stop flag pattern that works well with the chunked execution model.

**Primary recommendation:** Use chunked main-thread execution with `AtomicBool` cancellation flag. For macros with delays, coordinate timing in a background thread but execute keystrokes on main thread via channel commands. Register a "stop macro" hotkey that sets the cancellation flag.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| std::thread | (std) | Background thread for timing | No runtime dependency, simple API |
| std::sync::atomic::AtomicBool | (std) | Cancellation flag | Lock-free, Send+Sync, minimal overhead |
| std::sync::Arc | (std) | Shared ownership across threads | Standard pattern for shared atomic state |
| crossbeam-channel | 0.5 | MPSC communication | Already in Cargo.toml, high-performance, select! macro |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| winit EventLoopProxy | 0.30 | Wake main thread from worker | Already in use for hotkey events |
| global-hotkey | 0.7 | Register stop hotkey | Already in use, add stop binding |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| std::thread | tokio spawn_blocking | Adds async runtime dependency, overkill for this use case |
| AtomicBool | CancellationToken (tokio-util) | Requires tokio runtime, more complex |
| crossbeam-channel | std::sync::mpsc | crossbeam has better performance and select! |
| Main-thread injection | Worker-thread Enigo | macOS TIS/TSM errors, undefined behavior |

**No new dependencies needed.** Everything required is either in std or already in Cargo.toml.

## Architecture Patterns

### Recommended Execution Flow
```
User presses hotkey
        |
        v
+-------------------+
| Main Thread       |
|                   |
| 1. Receive hotkey |
| 2. Start execution|
+--------+----------+
         |
         v
   (Has delay_ms > 0?)
    /            \
   NO             YES
   |               |
   v               v
+--------+    +---------+
|Fast    |    |Spawn    |
|Execute |    |Worker   |
+--------+    +----+----+
                   |
                   v
            +------+-------+
            | Worker Thread|
            | - Sleep for  |
            |   delay_ms   |
            | - Send cmd   |
            |   to main    |
            +--------------+
                   |
                   v (channel)
            +------+-------+
            | Main Thread  |
            | - Receive cmd|
            | - Execute key|
            | - Check stop |
            +--------------+
```

### Pattern 1: Fast Path (No Delay)
**What:** Execute macro directly on main thread in one call
**When to use:** `delay_ms == 0` (instant mode)
**Why works:** Fast enough not to block noticeably

```rust
// Current code already does this - no change needed for fast path
// The enigo.text() call is essentially instantaneous
if delay_ms == 0 {
    injector.execute_sequence(&segments, 0)?;
}
```

### Pattern 2: Chunked Execution with Yield (Preferred for Delays)
**What:** Execute one segment at a time, yielding to event loop between segments
**When to use:** Macros with delays that need responsive UI

```rust
// State machine approach for execution
pub struct MacroExecution {
    segments: Vec<MacroSegment>,
    current_index: usize,
    delay_ms: u64,
    last_execute: Option<std::time::Instant>,
    stop_flag: Arc<AtomicBool>,
}

impl MacroExecution {
    pub fn new(segments: Vec<MacroSegment>, delay_ms: u64, stop_flag: Arc<AtomicBool>) -> Self {
        Self {
            segments,
            current_index: 0,
            delay_ms,
            last_execute: None,
            stop_flag,
        }
    }

    /// Returns true if execution should continue, false if done or cancelled
    pub fn tick(&mut self, injector: &mut KeystrokeInjector) -> bool {
        // Check cancellation
        if self.stop_flag.load(Ordering::Relaxed) {
            return false;
        }

        // Check if delay has elapsed
        if let Some(last) = self.last_execute {
            if last.elapsed() < Duration::from_millis(self.delay_ms) {
                return true; // Not ready yet, continue waiting
            }
        }

        // Execute current segment
        if self.current_index < self.segments.len() {
            let segment = &self.segments[self.current_index];
            // Execute single segment
            let _ = injector.execute_single_segment(segment);
            self.current_index += 1;
            self.last_execute = Some(std::time::Instant::now());
            true
        } else {
            false // Done
        }
    }
}

// In about_to_wait:
if let Some(ref mut execution) = self.active_execution {
    if !execution.tick(&mut self.injector) {
        self.active_execution = None; // Execution complete
    }
}
```

### Pattern 3: Worker Thread with Main Thread Injection
**What:** Background thread handles timing, main thread handles injection
**When to use:** Long macros where chunked approach creates too much state

```rust
// Command enum for cross-thread communication
pub enum InjectionCommand {
    Execute(MacroSegment),
    Done,
}

// Worker thread function
fn macro_worker(
    segments: Vec<MacroSegment>,
    delay_ms: u64,
    stop_flag: Arc<AtomicBool>,
    command_tx: crossbeam_channel::Sender<InjectionCommand>,
) {
    for segment in segments {
        // Check stop flag before each segment
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        // Send command to main thread for execution
        if command_tx.send(InjectionCommand::Execute(segment)).is_err() {
            break; // Main thread gone
        }

        // Wait between keystrokes
        if delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(delay_ms));
        }
    }

    let _ = command_tx.send(InjectionCommand::Done);
}

// Main thread receives and executes
// In about_to_wait:
while let Ok(cmd) = self.injection_rx.try_recv() {
    match cmd {
        InjectionCommand::Execute(segment) => {
            if let Some(ref mut injector) = self.injector {
                let _ = injector.execute_single_segment(&segment);
            }
        }
        InjectionCommand::Done => {
            self.macro_running = false;
        }
    }
}
```

### Pattern 4: Stop Hotkey Registration
**What:** Dedicated global hotkey to cancel running macro
**When to use:** Always register alongside macros

```rust
// Reserved stop hotkey (e.g., Escape or Ctrl+Escape)
const STOP_HOTKEY: HotKey = HotKey::new(None, Code::Escape);

// In hotkey handler:
if hotkey_event.id == self.stop_hotkey_id {
    if let Some(ref stop_flag) = self.active_stop_flag {
        stop_flag.store(true, Ordering::Relaxed);
        println!("Macro execution stopped by user");
    }
}
```

### Anti-Patterns to Avoid
- **Creating Enigo on worker thread (macOS):** Causes TIS/TSM errors, undefined behavior
- **Blocking event loop with thread::sleep:** Freezes UI, defeats purpose
- **Using async runtime for this:** Adds complexity, not IO-bound work
- **Polling stop flag with tight loop:** Wastes CPU, use channel select or timed checks
- **Ignoring thread join:** Can leak threads on app exit

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-thread channel | Manual Arc<Mutex<VecDeque>> | crossbeam-channel | Lock-free, faster, select! support |
| Cancellation token | Custom struct with Mutex<bool> | Arc<AtomicBool> | Lock-free, standard pattern |
| Thread-safe reference counting | Rc + unsafe | Arc | Atomic, safe, standard |
| Event loop wakeup | Custom pipe/socket | EventLoopProxy | Already using winit, integrated |
| Thread spawning | Manual OS threads | std::thread::spawn | Safe, manages resources |

**Key insight:** The Rust standard library's threading primitives (std::thread, AtomicBool, Arc) combined with crossbeam-channel provide everything needed. Adding an async runtime (tokio) would be overkill for this use case and complicate the architecture.

## Common Pitfalls

### Pitfall 1: Enigo on Background Thread (macOS)
**What goes wrong:** Console spam with "TIS/TSM in non-main thread environment" errors, potential crashes
**Why it happens:** macOS Core Graphics event APIs require main thread
**How to avoid:** ALWAYS execute Enigo calls on main thread; use channel to send commands from worker
**Warning signs:** Error messages mentioning TIS/TSM, keystroke injection fails silently on macOS

### Pitfall 2: Thread Not Joining on Exit
**What goes wrong:** Background thread continues after main thread exits, or crashes
**Why it happens:** std::thread::spawn creates detached threads by default
**How to avoid:** Store JoinHandle, join on app quit or use scoped threads
**Warning signs:** "thread panicked while panicking" on exit, memory leaks

### Pitfall 3: Stop Flag Never Checked
**What goes wrong:** User presses stop but macro continues to completion
**Why it happens:** Stop flag checked only at start, not between segments
**How to avoid:** Check stop flag before EVERY segment execution
**Warning signs:** Long macros can't be interrupted

### Pitfall 4: Memory Ordering Issues
**What goes wrong:** Stop flag change not visible across threads
**Why it happens:** Using wrong Ordering (though Relaxed is fine for stop flags)
**How to avoid:** Use Ordering::Relaxed for simple flags - it's sufficient for stop signals
**Warning signs:** Inconsistent cancellation behavior (rare with Relaxed, but possible with other orderings)

### Pitfall 5: Channel Deadlock
**What goes wrong:** Worker blocks on send, main blocks on recv, deadlock
**Why it happens:** Bounded channel full, receiver not draining
**How to avoid:** Use unbounded channel for injection commands, or try_send with timeout
**Warning signs:** App freezes, both threads stuck

### Pitfall 6: Flash Icon During Background Execution
**What goes wrong:** Icon flash happens before macro completes
**Why it happens:** Flash triggered immediately, not waiting for execution completion
**How to avoid:** Trigger flash in InjectionCommand::Done handler
**Warning signs:** Icon flashes before text appears

## Code Examples

Verified patterns from official sources and standard Rust idioms:

### Complete Async Execution Module
```rust
// src/execution.rs - New module for async macro execution

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crossbeam_channel::{Sender, Receiver, unbounded};

use crate::injection::{KeystrokeInjector, MacroSegment};

/// Command sent from worker thread to main thread
#[derive(Debug)]
pub enum ExecutionCommand {
    /// Execute a single segment on main thread
    Inject(MacroSegment),
    /// Execution completed successfully
    Complete,
    /// Execution was cancelled
    Cancelled,
}

/// Handle for controlling a running macro execution
pub struct ExecutionHandle {
    /// Set to true to request cancellation
    pub stop_flag: Arc<AtomicBool>,
    /// Thread handle for cleanup
    thread: Option<std::thread::JoinHandle<()>>,
}

impl ExecutionHandle {
    /// Request the execution to stop
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    /// Check if still running
    pub fn is_running(&self) -> bool {
        self.thread.as_ref().map_or(false, |t| !t.is_finished())
    }

    /// Wait for thread to complete (call on app exit)
    pub fn join(mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Start async execution of a macro
///
/// Returns a command receiver for the main thread and a handle for control.
/// Commands should be processed in the main event loop.
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

fn execution_worker(
    segments: Vec<MacroSegment>,
    delay_ms: u64,
    stop_flag: Arc<AtomicBool>,
    tx: Sender<ExecutionCommand>,
) {
    for segment in segments {
        // Check for cancellation before each segment
        if stop_flag.load(Ordering::Relaxed) {
            let _ = tx.send(ExecutionCommand::Cancelled);
            return;
        }

        // Send segment to main thread for execution
        if tx.send(ExecutionCommand::Inject(segment)).is_err() {
            // Receiver dropped, exit gracefully
            return;
        }

        // Wait between keystrokes if delay specified
        if delay_ms > 0 {
            // Use small sleep intervals to check stop flag more frequently
            let check_interval = Duration::from_millis(50.min(delay_ms));
            let total_delay = Duration::from_millis(delay_ms);
            let start = Instant::now();

            while start.elapsed() < total_delay {
                if stop_flag.load(Ordering::Relaxed) {
                    let _ = tx.send(ExecutionCommand::Cancelled);
                    return;
                }
                std::thread::sleep(check_interval);
            }
        }
    }

    let _ = tx.send(ExecutionCommand::Complete);
}
```

### Integration with Main Event Loop
```rust
// In KeyBlastApp struct:
struct KeyBlastApp {
    // ... existing fields ...

    /// Active execution handle (if macro running)
    active_execution: Option<ExecutionHandle>,
    /// Receiver for execution commands
    execution_rx: Option<Receiver<ExecutionCommand>>,
    /// ID of stop hotkey
    stop_hotkey_id: Option<u32>,
}

// In about_to_wait:
fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    // Process execution commands (non-blocking)
    if let Some(ref rx) = self.execution_rx {
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                ExecutionCommand::Inject(segment) => {
                    if let Some(ref mut injector) = self.injector {
                        // Execute on main thread - safe for macOS
                        let _ = injector.execute_single_segment(&segment);
                    }
                }
                ExecutionCommand::Complete => {
                    println!("Macro execution complete");
                    self.active_execution = None;
                    self.execution_rx = None;
                    // Trigger icon flash
                    self.flash_remaining = 4;
                }
                ExecutionCommand::Cancelled => {
                    println!("Macro execution cancelled");
                    self.active_execution = None;
                    self.execution_rx = None;
                }
            }
        }
    }

    // ... rest of about_to_wait ...
}

// In hotkey handler:
fn handle_hotkey(&mut self, hotkey_event: GlobalHotKeyEvent) {
    // Check for stop hotkey
    if Some(hotkey_event.id) == self.stop_hotkey_id {
        if let Some(ref handle) = self.active_execution {
            handle.stop();
        }
        return;
    }

    // Check if already executing
    if self.active_execution.is_some() {
        println!("Macro already running, ignoring new trigger");
        return;
    }

    // Look up and execute macro
    if let Some(macro_def) = self.macros.get(&hotkey_event.id) {
        let segments = parse_macro_sequence(&macro_def.text);

        if macro_def.delay_ms == 0 {
            // Fast path: execute immediately on main thread
            if let Some(ref mut injector) = self.injector {
                let _ = injector.execute_sequence(&segments, 0);
                self.flash_remaining = 4;
            }
        } else {
            // Async path: spawn worker thread
            let (rx, handle) = execution::start_execution(
                segments,
                macro_def.delay_ms,
            );
            self.execution_rx = Some(rx);
            self.active_execution = Some(handle);
        }
    }
}
```

### Menu Item for Stop Action
```rust
// Add "Stop Macro" menu item that's only enabled when macro running
let stop_item = MenuItem::new(
    "Stop Macro (Esc)",
    self.active_execution.is_some(),  // enabled only when running
    Some(Accelerator::new(None, Code::Escape)),
);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Enigo non-Send on macOS | Enigo 0.4+ implements Send/Sync | 2024 | Still requires main thread for TIS/TSM |
| async-std runtime | smol (lightweight) or just std::thread | 2025 | async-std deprecated |
| Manual channel impl | crossbeam-channel | Stable | Better performance, select! |
| Tokio for everything | std::thread for simple cases | Ongoing | Avoid runtime overhead when not needed |

**Current best practice:** For UI applications with simple background work, prefer std::thread over async runtimes. Reserve async for IO-bound work with many concurrent operations.

## Open Questions

Things that couldn't be fully resolved:

1. **Escape key conflict**
   - What we know: Escape is intuitive stop key, but user macros might use it
   - What's unclear: Best default stop hotkey that won't conflict
   - Recommendation: Use `Ctrl+Escape` or `Cmd+.` (macOS convention for cancel)

2. **Execution queue vs. ignore**
   - What we know: If macro running and user triggers another, should queue or ignore?
   - What's unclear: User expectation
   - Recommendation: Ignore with message; queuing adds complexity and may not match intent

3. **Stop menu item update**
   - What we know: Menu item needs to show enabled/disabled based on execution state
   - What's unclear: Whether muda supports dynamic enable/disable after menu creation
   - Recommendation: Test with `MenuItemKind::Normal().set_enabled()`; may need menu rebuild

## Sources

### Primary (HIGH confidence)
- [Rust std::thread documentation](https://doc.rust-lang.org/std/thread/) - Thread spawning, JoinHandle
- [Rust std::sync::atomic](https://doc.rust-lang.org/std/sync/atomic/) - AtomicBool, Ordering
- [crossbeam-channel docs](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/) - Channel API, select!
- [winit EventLoopProxy](https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopProxy.html) - Cross-thread wakeup

### Secondary (MEDIUM confidence)
- [enigo GitHub issue #96](https://github.com/enigo-rs/enigo/issues/96) - macOS thread safety confirmed
- [Apple Developer Forums](https://developer.apple.com/forums/thread/105244) - TIS/TSM main thread requirement
- [Rust Atomics and Locks book](https://marabos.nl/atomics/atomics.html) - AtomicBool stop flag pattern

### Tertiary (LOW confidence)
- Various blog posts on Rust threading patterns - General guidance, not library-specific

## Metadata

**Confidence breakdown:**
- Threading model: HIGH - Well-documented Rust std, confirmed macOS constraints
- Cancellation pattern: HIGH - Standard AtomicBool pattern, widely used
- Cross-thread communication: HIGH - crossbeam-channel is battle-tested
- Integration approach: MEDIUM - Architecture is sound but implementation details need verification

**Research date:** 2026-01-17
**Valid until:** 90 days (std library stable, crossbeam-channel mature)
