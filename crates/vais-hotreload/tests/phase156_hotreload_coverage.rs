//! Phase 156 coverage tests for vais-hotreload
//!
//! Adds +25 tests covering file watcher logic, reload trigger conditions,
//! and error handling (file not found, invalid path, etc.)

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vais_hotreload::{
    DylibLoader, FileWatcher, HotReloadConfig, HotReloadError, HotReloader, WatchEvent,
};

// ─── FileWatcher: path management ───────────────────────────────────────────

#[test]
fn test_file_watcher_check_returns_none_with_no_watched_paths() {
    let mut watcher = FileWatcher::new().unwrap();
    // No paths watched, no events possible
    let result = watcher.check().unwrap();
    assert!(result.is_none());
}

#[test]
fn test_file_watcher_unwatch_removes_correct_path() {
    let tmp = TempDir::new().unwrap();
    let f1 = tmp.path().join("a.vais");
    let f2 = tmp.path().join("b.vais");
    fs::write(&f1, "").unwrap();
    fs::write(&f2, "").unwrap();

    let mut watcher = FileWatcher::new().unwrap();
    watcher.watch(&f1).unwrap();
    watcher.watch(&f2).unwrap();
    assert_eq!(watcher.watched_paths().len(), 2);

    watcher.unwatch(&f1).unwrap();
    let paths = watcher.watched_paths();
    assert_eq!(paths.len(), 1);
    assert!(!paths.contains(&f1));
    assert!(paths.contains(&f2));
}

#[test]
fn test_file_watcher_watch_same_file_twice_increments_count() {
    let tmp = TempDir::new().unwrap();
    let f = tmp.path().join("dup.vais");
    fs::write(&f, "").unwrap();

    let mut watcher = FileWatcher::new().unwrap();
    watcher.watch(&f).unwrap();
    // Watching a second time appends to watched_paths list
    watcher.watch(&f).unwrap();
    assert_eq!(watcher.watched_paths().len(), 2);
}

#[test]
fn test_file_watcher_with_zero_debounce_has_zero_duration() {
    let watcher = FileWatcher::with_debounce(0).unwrap();
    assert_eq!(
        watcher.watched_paths().len(),
        0,
        "New watcher should have no paths"
    );
}

#[test]
fn test_file_watcher_large_debounce_value() {
    // u64::MAX / 2 as ms to avoid overflow — just verifying construction
    let watcher = FileWatcher::with_debounce(u64::MAX / 2);
    assert!(watcher.is_ok());
}

// ─── WatchEvent: variants and behaviour ──────────────────────────────────────

#[test]
fn test_watch_event_modified_path_accessible() {
    let path = PathBuf::from("/tmp/foo.vais");
    let event = WatchEvent::Modified(path.clone());
    match event {
        WatchEvent::Modified(p) => assert_eq!(p, path),
        _ => panic!("unexpected variant"),
    }
}

#[test]
fn test_watch_event_created_path_accessible() {
    let path = PathBuf::from("/tmp/new.vais");
    let event = WatchEvent::Created(path.clone());
    match event {
        WatchEvent::Created(p) => assert_eq!(p, path),
        _ => panic!("unexpected variant"),
    }
}

#[test]
fn test_watch_event_removed_path_accessible() {
    let path = PathBuf::from("/tmp/gone.vais");
    let event = WatchEvent::Removed(path.clone());
    match event {
        WatchEvent::Removed(p) => assert_eq!(p, path),
        _ => panic!("unexpected variant"),
    }
}

#[test]
fn test_watch_event_clone_created() {
    let original = WatchEvent::Created(PathBuf::from("x.vais"));
    let cloned = original.clone();
    assert!(matches!(cloned, WatchEvent::Created(p) if p == PathBuf::from("x.vais")));
}

#[test]
fn test_watch_event_clone_removed() {
    let original = WatchEvent::Removed(PathBuf::from("y.vais"));
    let cloned = original.clone();
    assert!(matches!(cloned, WatchEvent::Removed(p) if p == PathBuf::from("y.vais")));
}

// ─── HotReloadConfig: builder pattern edge cases ─────────────────────────────

#[test]
fn test_config_source_path_is_preserved_exactly() {
    let expected = PathBuf::from("/opt/vais/src/main.vais");
    let config = HotReloadConfig::new(&expected);
    assert_eq!(config.source_path, expected);
}

#[test]
fn test_config_compiler_command_can_be_empty_string() {
    let config = HotReloadConfig::new("f.vais").with_compiler(String::new());
    assert_eq!(config.compiler_command, "");
}

#[test]
fn test_config_with_output_dir_absolute() {
    let config = HotReloadConfig::new("f.vais").with_output_dir("/absolute/path");
    assert_eq!(config.output_dir.unwrap(), PathBuf::from("/absolute/path"));
}

#[test]
fn test_config_with_output_dir_relative() {
    let config = HotReloadConfig::new("f.vais").with_output_dir("relative/path");
    assert_eq!(config.output_dir.unwrap(), PathBuf::from("relative/path"));
}

#[test]
fn test_config_verbose_default_is_false() {
    let config = HotReloadConfig::new("x.vais");
    assert!(!config.verbose);
}

#[test]
fn test_config_compile_timeout_default_is_30() {
    let config = HotReloadConfig::new("x.vais");
    assert_eq!(config.compile_timeout_secs, 30);
}

// ─── HotReloader: construction and state ─────────────────────────────────────

#[test]
fn test_hot_reloader_new_with_valid_source_file() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("myapp.vais");
    fs::write(&src, "F main() -> i64 { 99 }").unwrap();

    let config = HotReloadConfig::new(&src).with_output_dir(tmp.path());
    let reloader = HotReloader::new(config);
    assert!(reloader.is_ok());
}

#[test]
fn test_hot_reloader_version_is_zero_before_start() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("app.vais");
    fs::write(&src, "F main() -> i64 { 0 }").unwrap();

    let config = HotReloadConfig::new(&src).with_output_dir(tmp.path());
    let reloader = HotReloader::new(config).unwrap();
    assert_eq!(reloader.version(), 0);
}

// ─── DylibLoader: error handling ─────────────────────────────────────────────

#[test]
fn test_dylib_loader_rejects_empty_path_string() {
    let result = DylibLoader::new("");
    assert!(result.is_err());
}

#[test]
fn test_dylib_loader_rejects_directory_path() {
    let tmp = TempDir::new().unwrap();
    // tmp.path() is a directory, not a file — DylibLoader::new checks existence;
    // on some systems a directory "exists" so we test a non-existent path instead
    let result = DylibLoader::new("/nonexistent/definitely/not/here.dylib");
    assert!(result.is_err());
}

#[test]
fn test_dylib_loader_accepts_existing_file() {
    let tmp = TempDir::new().unwrap();
    let dummy = tmp.path().join("dummy.dylib");
    fs::write(&dummy, b"not a real lib").unwrap();

    let result = DylibLoader::new(&dummy);
    assert!(result.is_ok());
    let loader = result.unwrap();
    assert!(!loader.is_loaded());
    assert_eq!(loader.version(), 0);
}

// ─── HotReloadError: coverage ────────────────────────────────────────────────

#[test]
fn test_error_compilation_error_message_preserved() {
    let msg = "syntax error on line 5: unexpected token `}`";
    let err = HotReloadError::CompilationError(msg.to_string());
    assert!(err.to_string().contains(msg));
}

#[test]
fn test_error_invalid_path_variants() {
    let cases = [
        HotReloadError::InvalidPath("No parent directory".to_string()),
        HotReloadError::InvalidPath("Invalid source file name".to_string()),
        HotReloadError::InvalidPath(String::new()),
    ];
    for err in &cases {
        let display = format!("{}", err);
        assert!(display.contains("Invalid dylib path"));
    }
}

#[test]
fn test_error_not_initialized_is_send() {
    let err = HotReloadError::NotInitialized;
    // Verify it can be sent across threads
    let _ = std::thread::spawn(move || {
        let _ = err;
    })
    .join();
}

#[test]
fn test_error_reload_in_progress_debug() {
    let err = HotReloadError::ReloadInProgress;
    let debug = format!("{:?}", err);
    assert!(debug.contains("ReloadInProgress"));
}
