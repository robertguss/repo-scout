pub mod launch {
    pub const DEFAULT_RETRIES: usize = 3;

    pub type LaunchId = u64;

    pub struct Launcher {
        pub id: LaunchId,
    }

    pub enum LaunchState {
        Pending,
        Running,
        Finished,
    }

    pub trait Runnable {
        fn run(&self);
    }

    impl Runnable for Launcher {
        /// Starts the engine associated with this launcher.
        ///
        /// # Examples
        ///
        /// ```
        /// let launcher = crate::launch::Launcher { id: 7 };
        /// launcher.run();
        /// ```
        fn run(&self) {
            start_engine(self.id);
        }
    }

    /// Starts the engine associated with the given launch identifier.
///
/// # Examples
///
/// ```
/// // Call with a LaunchId (u64)
/// launch::start_engine(7);
/// ```
pub fn start_engine(_id: LaunchId) {}
}

use launch::Launcher;

/// Constructs a `Launcher` with its `id` set to `7`.
///
/// The returned `Launcher` will have `id == 7`.
///
/// # Examples
///
/// ```
/// let l = crate::make_launcher();
/// assert_eq!(l.id, 7);
/// ```
pub fn make_launcher() -> Launcher {
    Launcher { id: 7 }
}