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

// HotReloadConfig advanced tests (3)

#[test]
fn test_hot_reload_config_with_verbose() {
    let config = HotReloadConfig::new("test.vais").with_verbose(true);
    assert!(config.verbose);

    let config2 = HotReloadConfig::new("test.vais").with_verbose(false);
    assert!(!config2.verbose);
}

#[test]
fn test_hot_reload_config_compile_timeout() {
    let config = HotReloadConfig::new("test.vais");
    assert_eq!(config.compile_timeout_secs, 30);

    // Note: compile_timeout_secs has no setter in public API
    // This test just verifies the default value
}

#[test]
fn test_hot_reload_config_all_methods_chained() {
    let config = HotReloadConfig::new("game.vais")
        .with_output_dir("/tmp/output")
        .with_compiler("custom-compiler".to_string())
        .with_compiler_args(vec!["--opt".to_string(), "--debug".to_string()])
        .with_debounce(500)
        .with_verbose(true);

    assert_eq!(config.source_path.to_str().unwrap(), "game.vais");
    assert_eq!(config.output_dir.unwrap().to_str().unwrap(), "/tmp/output");
    assert_eq!(config.compiler_command, "custom-compiler");
    assert_eq!(config.compiler_args, vec!["--opt", "--debug"]);
    assert_eq!(config.debounce_ms, 500);
    assert!(config.verbose);
}

// FileWatcher advanced tests (3)

#[test]
fn test_file_watcher_watch_unwatch_cycle() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("cycle.vais");
    fs::write(&test_file, "F main() -> i64 { 1 }").unwrap();

    let mut watcher = FileWatcher::new().unwrap();

    // Watch
    assert!(watcher.watch(&test_file).is_ok());
    assert_eq!(watcher.watched_paths().len(), 1);

    // Unwatch
    assert!(watcher.unwatch(&test_file).is_ok());
    assert_eq!(watcher.watched_paths().len(), 0);

    // Watch again
    assert!(watcher.watch(&test_file).is_ok());
    assert_eq!(watcher.watched_paths().len(), 1);
}

#[test]
fn test_file_watcher_watched_paths_verification() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("file1.vais");
    let file2 = temp_dir.path().join("file2.vais");

    fs::write(&file1, "F foo() -> i64 { 1 }").unwrap();
    fs::write(&file2, "F bar() -> i64 { 2 }").unwrap();

    let mut watcher = FileWatcher::new().unwrap();
    watcher.watch(&file1).unwrap();
    watcher.watch(&file2).unwrap();

    let watched = watcher.watched_paths();
    assert_eq!(watched.len(), 2);
    assert!(watched.contains(&file1));
    assert!(watched.contains(&file2));
}

#[test]
fn test_file_watcher_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let files: Vec<_> = (0..3)
        .map(|i| {
            let path = temp_dir.path().join(format!("file{}.vais", i));
            fs::write(&path, format!("F func{}() -> i64 {{ {} }}", i, i)).unwrap();
            path
        })
        .collect();

    let mut watcher = FileWatcher::new().unwrap();

    for file in &files {
        assert!(watcher.watch(file).is_ok());
    }

    assert_eq!(watcher.watched_paths().len(), 3);

    // Unwatch one file
    watcher.unwatch(&files[1]).unwrap();
    assert_eq!(watcher.watched_paths().len(), 2);
}

// DylibLoader tests (2)

#[test]
fn test_dylib_loader_nonexistent_file_error() {
    use vais_hotreload::DylibLoader;

    let result = DylibLoader::new("/nonexistent/path/to/lib.dylib");
    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("Dylib not found") || error_msg.contains("Invalid"));
    }
}

#[test]
fn test_dylib_loader_initial_state() {
    use std::fs::File;
    use vais_hotreload::DylibLoader;

    let temp_dir = TempDir::new().unwrap();
    let dummy_lib = temp_dir.path().join("libdummy.dylib");

    // Create a dummy file (not a real dylib, but exists)
    File::create(&dummy_lib).unwrap();

    let loader = DylibLoader::new(&dummy_lib).unwrap();

    // Check initial state
    assert_eq!(loader.version(), 0);
    assert!(!loader.is_loaded());
}

// Error types test (1)

#[test]
fn test_hot_reload_error_variants() {
    use vais_hotreload::HotReloadError;

    // Test each error variant can be created and Display works
    let errors = vec![
        HotReloadError::CompilationError("test compilation error".to_string()),
        HotReloadError::SymbolNotFound("missing_symbol".to_string()),
        HotReloadError::InvalidPath("/invalid/path".to_string()),
        HotReloadError::NotInitialized,
        HotReloadError::ReloadInProgress,
        HotReloadError::CompilationTimeout,
    ];

    for error in errors {
        let display = format!("{}", error);
        assert!(!display.is_empty());

        // Verify error messages contain expected text
        match error {
            HotReloadError::CompilationError(ref msg) => {
                assert!(display.contains("Compilation error"));
                assert!(display.contains(msg));
            }
            HotReloadError::SymbolNotFound(ref name) => {
                assert!(display.contains("Symbol not found"));
                assert!(display.contains(name));
            }
            HotReloadError::InvalidPath(ref path) => {
                assert!(display.contains("Invalid dylib path"));
                assert!(display.contains(path));
            }
            HotReloadError::NotInitialized => {
                assert!(display.contains("not initialized"));
            }
            HotReloadError::ReloadInProgress => {
                assert!(display.contains("in progress"));
            }
            HotReloadError::CompilationTimeout => {
                assert!(display.contains("Timeout"));
            }
            _ => {}
        }
    }
}

// HotReloader edge cases test (1)

#[test]
fn test_hot_reloader_new_config_validation() {
    use vais_hotreload::HotReloader;

    let temp_dir = TempDir::new().unwrap();
    let source_file = temp_dir.path().join("app.vais");
    fs::write(&source_file, "F main() -> i64 { 42 }").unwrap();

    let config = HotReloadConfig::new(&source_file)
        .with_output_dir(temp_dir.path())
        .with_debounce(150);

    // Create HotReloader and verify it accepts valid config
    let reloader = HotReloader::new(config);
    assert!(reloader.is_ok());

    let reloader = reloader.unwrap();
    assert_eq!(reloader.version(), 0);
}
