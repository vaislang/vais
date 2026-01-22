//! Vais Hot Reloading System
//!
//! Provides runtime code reloading capabilities for Vais programs.
//!
//! # Architecture
//!
//! The hot reload system consists of three main components:
//!
//! - **FileWatcher**: Monitors source files for changes using the `notify` crate
//! - **DylibLoader**: Dynamically loads and unloads shared libraries using `libloading`
//! - **HotReloader**: Coordinates file watching and library reloading
//!
//! # Usage
//!
//! ```ignore
//! use vais_hotreload::{HotReloader, HotReloadConfig};
//!
//! let config = HotReloadConfig::new("./game.vais");
//! let mut reloader = HotReloader::new(config)?;
//!
//! // Start watching for changes
//! reloader.start()?;
//!
//! // In your main loop
//! loop {
//!     if reloader.check()? {
//!         println!("Code reloaded!");
//!     }
//!     // ... run your game logic
//! }
//! ```

mod file_watcher;
mod dylib_loader;
mod reloader;
mod error;

pub use file_watcher::{FileWatcher, WatchEvent};
pub use dylib_loader::{DylibLoader, FunctionSymbol};
pub use reloader::{HotReloader, HotReloadConfig, ReloadCallback};
pub use error::{HotReloadError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_config() {
        let config = HotReloadConfig::new("test.vais");
        assert_eq!(config.source_path.to_str().unwrap(), "test.vais");
    }
}
