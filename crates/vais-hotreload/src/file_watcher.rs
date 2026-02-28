//! File system watching for source file changes

use crate::error::{HotReloadError, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, SystemTime};

/// Events emitted by the file watcher
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// File was modified
    Modified(PathBuf),
    /// File was created
    Created(PathBuf),
    /// File was removed
    Removed(PathBuf),
}

/// Watches source files for changes
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
    watched_paths: Vec<PathBuf>,
    last_event_time: Option<SystemTime>,
    debounce_duration: Duration,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self> {
        let (tx, rx) = channel();
        let watcher = Self::create_watcher(tx)?;

        Ok(FileWatcher {
            watcher,
            receiver: rx,
            watched_paths: Vec::new(),
            last_event_time: None,
            debounce_duration: Duration::from_millis(100),
        })
    }

    /// Create a watcher with custom debounce duration
    pub fn with_debounce(debounce_ms: u64) -> Result<Self> {
        let mut watcher = Self::new()?;
        watcher.debounce_duration = Duration::from_millis(debounce_ms);
        Ok(watcher)
    }

    /// Add a path to watch
    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        self.watcher.watch(path, RecursiveMode::NonRecursive)?;
        self.watched_paths.push(path.to_path_buf());
        Ok(())
    }

    /// Stop watching a path
    pub fn unwatch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        self.watcher.unwatch(path)?;
        self.watched_paths.retain(|p| p != path);
        Ok(())
    }

    /// Check for file changes (non-blocking)
    /// Returns None if no changes, Some(WatchEvent) if a change was detected
    pub fn check(&mut self) -> Result<Option<WatchEvent>> {
        // Drain all pending events
        let mut last_event: Option<WatchEvent> = None;

        while let Ok(result) = self.receiver.try_recv() {
            match result {
                Ok(event) => {
                    if let Some(watch_event) = self.process_event(event)? {
                        // Check debouncing
                        let now = SystemTime::now();
                        if let Some(last_time) = self.last_event_time {
                            if now.duration_since(last_time).unwrap_or(Duration::ZERO)
                                < self.debounce_duration
                            {
                                continue; // Skip this event due to debouncing
                            }
                        }

                        self.last_event_time = Some(now);
                        last_event = Some(watch_event);
                    }
                }
                Err(e) => return Err(HotReloadError::WatchError(e)),
            }
        }

        Ok(last_event)
    }

    /// Wait for a file change (blocking)
    pub fn wait(&mut self) -> Result<WatchEvent> {
        loop {
            match self.receiver.recv() {
                Ok(result) => match result {
                    Ok(event) => {
                        if let Some(watch_event) = self.process_event(event)? {
                            // Check debouncing
                            let now = SystemTime::now();
                            if let Some(last_time) = self.last_event_time {
                                if now.duration_since(last_time).unwrap_or(Duration::ZERO)
                                    < self.debounce_duration
                                {
                                    continue; // Skip this event due to debouncing
                                }
                            }

                            self.last_event_time = Some(now);
                            return Ok(watch_event);
                        }
                    }
                    Err(e) => return Err(HotReloadError::WatchError(e)),
                },
                Err(_) => {
                    return Err(HotReloadError::WatchError(notify::Error::generic(
                        "watcher channel closed",
                    )))
                }
            }
        }
    }

    /// Get the list of watched paths
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }

    fn create_watcher(tx: Sender<notify::Result<Event>>) -> Result<RecommendedWatcher> {
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            notify::Config::default(),
        )?;
        Ok(watcher)
    }

    fn process_event(&self, event: Event) -> Result<Option<WatchEvent>> {
        let watch_event = match event.kind {
            EventKind::Modify(_) => {
                if let Some(path) = event.paths.first() {
                    // Only report modifications for watched .vais files
                    if self.is_watched_file(path) {
                        Some(WatchEvent::Modified(path.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            EventKind::Create(_) => {
                if let Some(path) = event.paths.first() {
                    if self.is_watched_file(path) {
                        Some(WatchEvent::Created(path.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            EventKind::Remove(_) => event
                .paths
                .first()
                .map(|path| WatchEvent::Removed(path.clone())),
            _ => None,
        };

        Ok(watch_event)
    }

    fn is_watched_file(&self, path: &Path) -> bool {
        // Check if this is a .vais file or a dylib that we're watching
        self.watched_paths.iter().any(|p| p == path)
            || path.extension().is_some_and(|ext| ext == "vais")
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().expect("failed to create file watcher")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_watcher_creation() {
        let watcher = FileWatcher::new();
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_file_watcher_default() {
        let watcher = FileWatcher::default();
        assert!(watcher.watched_paths().is_empty());
        assert_eq!(watcher.debounce_duration, Duration::from_millis(100));
    }

    #[test]
    fn test_watch_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.vais");
        fs::write(&test_file, "F main() -> i64 { 0 }").unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        assert!(watcher.watch(&test_file).is_ok());
        assert_eq!(watcher.watched_paths().len(), 1);
    }

    #[test]
    fn test_watch_multiple_paths() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("a.vais");
        let file2 = temp_dir.path().join("b.vais");
        fs::write(&file1, "F a() -> i64 { 0 }").unwrap();
        fs::write(&file2, "F b() -> i64 { 0 }").unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        assert!(watcher.watch(&file1).is_ok());
        assert!(watcher.watch(&file2).is_ok());
        assert_eq!(watcher.watched_paths().len(), 2);
    }

    #[test]
    fn test_unwatch_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.vais");
        fs::write(&test_file, "F main() -> i64 { 0 }").unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        watcher.watch(&test_file).unwrap();
        assert_eq!(watcher.watched_paths().len(), 1);

        watcher.unwatch(&test_file).unwrap();
        assert_eq!(watcher.watched_paths().len(), 0);
    }

    #[test]
    fn test_debounce() {
        let watcher = FileWatcher::with_debounce(200);
        assert!(watcher.is_ok());
        let watcher = watcher.unwrap();
        assert_eq!(watcher.debounce_duration, Duration::from_millis(200));
    }

    #[test]
    fn test_debounce_zero() {
        let watcher = FileWatcher::with_debounce(0).unwrap();
        assert_eq!(watcher.debounce_duration, Duration::from_millis(0));
    }

    #[test]
    fn test_default_debounce() {
        let watcher = FileWatcher::new().unwrap();
        assert_eq!(watcher.debounce_duration, Duration::from_millis(100));
    }

    #[test]
    fn test_watched_paths_empty_initially() {
        let watcher = FileWatcher::new().unwrap();
        assert!(watcher.watched_paths().is_empty());
    }

    #[test]
    fn test_check_no_events() {
        let mut watcher = FileWatcher::new().unwrap();
        let result = watcher.check().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_watch_event_modified_debug() {
        let event = WatchEvent::Modified(PathBuf::from("test.vais"));
        let debug = format!("{:?}", event);
        assert!(debug.contains("Modified"));
        assert!(debug.contains("test.vais"));
    }

    #[test]
    fn test_watch_event_created_debug() {
        let event = WatchEvent::Created(PathBuf::from("new.vais"));
        let debug = format!("{:?}", event);
        assert!(debug.contains("Created"));
        assert!(debug.contains("new.vais"));
    }

    #[test]
    fn test_watch_event_removed_debug() {
        let event = WatchEvent::Removed(PathBuf::from("old.vais"));
        let debug = format!("{:?}", event);
        assert!(debug.contains("Removed"));
        assert!(debug.contains("old.vais"));
    }

    #[test]
    fn test_watch_event_clone() {
        let event = WatchEvent::Modified(PathBuf::from("test.vais"));
        let cloned = event.clone();
        assert!(matches!(cloned, WatchEvent::Modified(p) if p == Path::new("test.vais")));
    }

    #[test]
    fn test_watched_paths_contains_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.vais");
        fs::write(&test_file, "F main() -> i64 { 0 }").unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        watcher.watch(&test_file).unwrap();
        assert!(watcher.watched_paths().contains(&test_file));
    }

    #[test]
    fn test_last_event_time_initially_none() {
        let watcher = FileWatcher::new().unwrap();
        assert!(watcher.last_event_time.is_none());
    }
}
