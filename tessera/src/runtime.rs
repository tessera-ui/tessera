//! # Tessera Runtime System
//!
//! This module provides the global runtime state management for the Tessera UI framework.
//! The runtime system maintains essential application state including the component tree,
//! window properties, and user interface state that needs to be shared across the entire
//! application lifecycle.
//!
//! ## Overview
//!
//! The [`TesseraRuntime`] serves as the central hub for all runtime data and side effects
//! in a Tessera application. It uses a thread-safe singleton pattern to ensure consistent
//! access to shared state from any part of the application, including from multiple threads
//! during parallel component processing.
//!
//! ## Thread Safety
//!
//! The runtime is designed with parallelization in mind. It uses [`parking_lot::RwLock`]
//! for efficient read-write synchronization, allowing multiple concurrent readers while
//! ensuring exclusive access for writers. This design supports Tessera's parallel
//! component tree processing capabilities.
//!
//! ## Usage
//!
//! Access the runtime through the static methods:
//!
//! ```
//! use tessera::TesseraRuntime;
//!
//! // Read-only access (multiple threads can read simultaneously)
//! {
//!     let runtime = TesseraRuntime::read();
//!     let window_size = runtime.window_size;
//!     println!("Window size: {}x{}", window_size[0], window_size[1]);
//! } // Lock is automatically released
//!
//! // Write access (exclusive access required)
//! {
//!     let mut runtime = TesseraRuntime::write();
//!     runtime.window_size = [1920, 1080];
//!     runtime.cursor_icon_request = Some(winit::window::CursorIcon::Pointer);
//! } // Lock is automatically released
//! ```
//!
//! ## Performance Considerations
//!
//! - Prefer read locks when only accessing data
//! - Keep lock scopes as narrow as possible to minimize contention
//! - The runtime is optimized for frequent reads and occasional writes
//! - Component tree operations may involve parallel processing under read locks

use std::sync::OnceLock;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::component_tree::ComponentTree;

/// Global singleton instance of the Tessera runtime.
///
/// This static variable ensures that there is exactly one runtime instance per application,
/// initialized lazily on first access. The [`OnceLock`] provides thread-safe initialization
/// without the overhead of synchronization after the first initialization.
static TESSERA_RUNTIME: OnceLock<RwLock<TesseraRuntime>> = OnceLock::new();

/// Central runtime state container for the Tessera UI framework.
///
/// The `TesseraRuntime` holds all global state and side effects that need to be shared
/// across the entire application. This includes the component tree structure, window
/// properties, and user interface state that persists across frame updates.
///
/// ## Design Philosophy
///
/// The runtime follows these key principles:
/// - **Single Source of Truth**: All shared state is centralized in one location
/// - **Thread Safety**: Safe concurrent access through read-write locks
/// - **Lazy Initialization**: Runtime is created only when first accessed
/// - **Minimal Overhead**: Optimized for frequent reads and occasional writes
///
/// ## Lifecycle
///
/// The runtime is automatically initialized on first access and persists for the
/// entire application lifetime. It cannot be manually destroyed or recreated.
///
/// ## Fields
///
/// All fields are public to allow direct access after acquiring the appropriate lock.
/// However, consider using higher-level APIs when available to maintain consistency.
#[derive(Default)]
pub struct TesseraRuntime {
    /// The hierarchical structure of all UI components in the application.
    ///
    /// This tree represents the current state of the UI hierarchy, including
    /// component relationships, layout information, and rendering data. The
    /// component tree is rebuilt or updated each frame during the UI update cycle.
    ///
    /// ## Thread Safety
    ///
    /// While the runtime itself is thread-safe, individual operations on the
    /// component tree may require coordination to maintain consistency during
    /// parallel processing phases.
    pub component_tree: ComponentTree,

    /// Current window dimensions in physical pixels.
    ///
    /// This array contains `[width, height]` representing the current size of
    /// the application window. These values are updated automatically when the
    /// window is resized and are used for layout calculations and rendering.
    ///
    /// ## Coordinate System
    ///
    /// - Values are in physical pixels (not density-independent pixels)
    /// - Origin is at the top-left corner of the window
    /// - Both dimensions are guaranteed to be non-negative
    pub window_size: [u32; 2],

    /// Cursor icon change request from UI components.
    ///
    /// Components can request cursor icon changes by setting this field during
    /// their update cycle. The windowing system will apply the requested cursor
    /// icon if present, or use the default cursor if `None`.
    ///
    /// ## Lifecycle
    ///
    /// This field is typically:
    /// 1. Reset to `None` at the beginning of each frame
    /// 2. Set by components during event handling or state updates
    /// 3. Applied by the windowing system at the end of the frame
    ///
    /// ## Priority
    ///
    /// If multiple components request different cursor icons in the same frame,
    /// the last request takes precedence. Components should coordinate cursor
    /// changes or use a priority system if needed.
    pub cursor_icon_request: Option<winit::window::CursorIcon>,
}

impl TesseraRuntime {
    /// Acquires shared read access to the runtime state.
    ///
    /// This method returns a read guard that allows concurrent access to the runtime
    /// data from multiple threads. Multiple readers can access the runtime simultaneously,
    /// but no writers can modify the state while any read guards exist.
    ///
    /// ## Blocking Behavior
    ///
    /// This method will block the current thread if a write lock is currently held.
    /// It will return immediately if no write locks are active, even if other read
    /// locks exist.
    ///
    /// ## Usage
    ///
    /// ```
    /// use tessera::TesseraRuntime;
    ///
    /// // Access runtime data for reading
    /// let runtime = TesseraRuntime::read();
    /// let [width, height] = runtime.window_size;
    /// println!("Window size: {}x{}", width, height);
    /// // Lock is automatically released when `runtime` goes out of scope
    /// ```
    ///
    /// ## Performance
    ///
    /// Read locks are optimized for high-frequency access and have minimal overhead
    /// when no write contention exists. Prefer read locks over write locks whenever
    /// possible to maximize parallelism.
    ///
    /// ## Deadlock Prevention
    ///
    /// To prevent deadlocks:
    /// - Always acquire locks in a consistent order
    /// - Keep lock scopes as narrow as possible
    /// - Avoid calling other locking functions while holding a lock
    ///
    /// # Returns
    ///
    /// A [`RwLockReadGuard`] that provides read-only access to the runtime state.
    /// The guard automatically releases the lock when dropped.
    pub fn read() -> RwLockReadGuard<'static, Self> {
        TESSERA_RUNTIME
            .get_or_init(|| RwLock::new(Self::default()))
            .read()
    }

    /// Acquires exclusive write access to the runtime state.
    ///
    /// This method returns a write guard that provides exclusive access to modify
    /// the runtime data. Only one writer can access the runtime at a time, and no
    /// readers can access the state while a write lock is held.
    ///
    /// ## Blocking Behavior
    ///
    /// This method will block the current thread until all existing read and write
    /// locks are released. It guarantees exclusive access once acquired.
    ///
    /// ## Usage
    ///
    /// ```
    /// use tessera::TesseraRuntime;
    ///
    /// // Modify runtime state
    /// {
    ///     let mut runtime = TesseraRuntime::write();
    ///     runtime.window_size = [1920, 1080];
    ///     runtime.cursor_icon_request = Some(winit::window::CursorIcon::Pointer);
    /// } // Lock is automatically released
    /// ```
    ///
    /// ## Performance Considerations
    ///
    /// Write locks are more expensive than read locks and should be used sparingly:
    /// - Batch multiple modifications into a single write lock scope
    /// - Release write locks as quickly as possible
    /// - Consider if the operation truly requires exclusive access
    ///
    /// ## Deadlock Prevention
    ///
    /// The same deadlock prevention guidelines apply as with [`read()`](Self::read):
    /// - Acquire locks in consistent order
    /// - Minimize lock scope duration
    /// - Avoid nested locking operations
    ///
    /// # Returns
    ///
    /// A [`RwLockWriteGuard`] that provides exclusive read-write access to the
    /// runtime state. The guard automatically releases the lock when dropped.
    pub fn write() -> RwLockWriteGuard<'static, Self> {
        TESSERA_RUNTIME
            .get_or_init(|| RwLock::new(Self::default()))
            .write()
    }
}
