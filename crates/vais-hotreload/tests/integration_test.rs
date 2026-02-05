use std::fs;
use tempfile::TempDir;
use vais_hotreload::{FileWatcher, HotReloadConfig};

#[test]
fn test_file_watcher_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.vais");

    // Create initial file
    fs::write(&test_file, "F main() -> i64 { 0 }").unwrap();

    // Create watcher
    let mut watcher = FileWatcher::new().unwrap();
    assert!(watcher.watch(&test_file).is_ok());

    // Check initial state
    assert_eq!(watcher.watched_paths().len(), 1);
}

#[test]
fn test_file_watcher_debounce() {
    let watcher = FileWatcher::with_debounce(200);
    assert!(watcher.is_ok());
}

#[test]
fn test_hot_reload_config_builder() {
    let config = HotReloadConfig::new("test.vais")
        .with_output_dir("/tmp")
        .with_compiler("custom-vaisc".to_string())
        .with_debounce(200)
        .with_verbose(true);

    assert_eq!(config.source_path.to_str().unwrap(), "test.vais");
    assert_eq!(config.output_dir.unwrap().to_str().unwrap(), "/tmp");
    assert_eq!(config.compiler_command, "custom-vaisc");
    assert_eq!(config.debounce_ms, 200);
    assert!(config.verbose);
}

#[test]
fn test_hot_reload_config_defaults() {
    let config = HotReloadConfig::new("test.vais");
    assert_eq!(config.compiler_command, "vaisc");
    assert_eq!(config.debounce_ms, 100);
    assert_eq!(config.compile_timeout_secs, 30);
    assert!(!config.verbose);
}

#[test]
fn test_dylib_path_determination() {
    let config = HotReloadConfig::new("/tmp/test.vais");
    // This would normally be tested with an actual HotReloader,
    // but we can verify the config is valid
    assert!(config.source_path.to_str().unwrap().ends_with("test.vais"));
}

#[test]
fn test_multiple_compiler_args() {
    let config = HotReloadConfig::new("test.vais")
        .with_compiler_args(vec!["-O2".to_string(), "--verbose".to_string()]);

    assert_eq!(config.compiler_args.len(), 2);
    assert_eq!(config.compiler_args[0], "-O2");
    assert_eq!(config.compiler_args[1], "--verbose");
}
