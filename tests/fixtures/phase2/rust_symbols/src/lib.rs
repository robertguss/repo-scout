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
        fn run(&self) {
            start_engine(self.id);
        }
    }

    pub fn start_engine(_id: LaunchId) {}
}

use launch::Launcher;

pub fn make_launcher() -> Launcher {
    Launcher { id: 7 }
}
