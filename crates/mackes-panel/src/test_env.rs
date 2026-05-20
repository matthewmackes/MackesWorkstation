//! Test-only helper: process-wide env mutex.
//!
//! Tests across `recents`, `desktop_files`, `notification_center`,
//! and friends all mutate `HOME` / `XDG_*` to redirect filesystem
//! lookups. Rust's test runner threads these tests in parallel, and
//! the env is a process-wide singleton — without serialization the
//! tests step on each other's vars and intermittently fail.
//!
//! Lock pattern:
//!
//! ```ignore
//! let _g = crate::test_env::env_lock();
//! std::env::set_var("HOME", path);
//! …
//! std::env::remove_var("HOME");
//! ```
//!
//! Holding the guard for the full test body keeps every read after
//! the set seeing the value we wrote.

use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Acquire the global env-mutating mutex. Poisoned guards are
/// ignored — a panic in one test must not block every later test.
pub fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Try to initialize GTK from a test. Returns `true` when init
/// succeeded (or was already initialized on this thread) and `false`
/// when no display is available — caller treats that as a skip
/// (`return;`).
///
/// `cargo test` is multi-threaded; GTK demands single-thread init.
/// We serialize every GTK-touching test through the process-wide
/// `env_lock` AND pin the init flag in a thread-local — within the
/// lock, `gtk::init()` is idempotent on the same thread, so a
/// subsequent call on a different worker thread silently skips.
///
/// Pairs with `env_lock()`:
///
/// ```ignore
/// let _g = crate::test_env::env_lock();
/// if !crate::test_env::try_init_gtk_serialized() {
///     return;
/// }
/// // …build widgets + assert on structure…
/// ```
///
/// Mirrors the pattern previously inlined in
/// `root_menu::tests::try_init_gtk_serialized`. Phase 9.2 widget
/// tests live across many modules, so we lifted the helper into
/// `test_env` rather than copy-pasting it module-by-module.
#[allow(dead_code)] // used by widget tests in sibling modules only
pub fn try_init_gtk_serialized() -> bool {
    use std::cell::Cell;
    thread_local! {
        static THIS_THREAD_INITED: Cell<bool> = const { Cell::new(false) };
    }
    static FIRST_THREAD: OnceLock<std::thread::ThreadId> = OnceLock::new();
    let current = std::thread::current().id();
    let owner = FIRST_THREAD.get_or_init(|| current);
    if *owner != current {
        return false;
    }
    if THIS_THREAD_INITED.with(Cell::get) {
        return true;
    }
    if gtk::init().is_err() {
        return false;
    }
    THIS_THREAD_INITED.with(|c| c.set(true));
    true
}
