use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn e2e_string_substring_returned_struct_field_per_module() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let helper_path = temp.path().join("string_owner.vais");
    let exe_path = temp.path().join("string_owner_smoke");

    std::fs::write(
        &main_path,
        r#"
use string_owner

fn main() -> i64 {
    box := make_sub_box("abcdef")
    check_sub_box(box)
}
"#,
    )
    .expect("write main fixture");
    std::fs::write(
        &helper_path,
        r#"
struct SubBox {
    key: str,
    value: str,
}

fn make_sub_box(source: str) -> SubBox {
    box := mut SubBox { key: "", value: "" }
    box.key = source.substring(0, 3)
    box.value = source.substring(3, 6)
    box
}

fn check_sub_box(box: SubBox) -> i64 {
    I box.key.len() != 3 { return 1 }
    I box.value.len() != 3 { return 2 }
    I box.key.char_at(0) != 97 { return 3 }
    I box.key.char_at(1) != 98 { return 4 }
    I box.key.char_at(2) != 99 { return 5 }
    I box.value.char_at(0) != 100 { return 6 }
    I box.value.char_at(1) != 101 { return 7 }
    I box.value.char_at(2) != 102 { return 8 }
    0
}
"#,
    )
    .expect("write helper fixture");

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", std_link(&compiler_root()))
        .env("VAIS_DEP_PATHS", temp.path())
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "string ownership fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .output()
        .expect("run string ownership fixture");
    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "string ownership fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn e2e_string_if_expr_retains_return_context_with_block_cleanup() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let main_path = temp.path().join("main.vais");
    let exe_path = temp.path().join("string_if_phi_smoke");

    std::fs::write(
        &main_path,
        r#"
struct Pair {
    key: str,
    value: str,
}

fn lookup(i: i64) -> str {
    I i >= 1 { "" }
    else {
        pair := Pair { key: "tab", value: "posts" }
        I pair.key == "tab" { pair.value }
        else { lookup(i + 1) }
    }
}

fn main() -> i64 {
    value := lookup(0)
    I value != "posts" { return 1 }
    0
}
"#,
    )
    .expect("write main fixture");

    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("build")
        .arg(&main_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .current_dir(temp.path())
        .env("VAIS_STD_PATH", std_link(&compiler_root()))
        .env("VAIS_DEP_PATHS", temp.path())
        .output()
        .expect("spawn vaisc build");
    assert!(
        build.status.success(),
        "string if-expression fixture failed to build\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .output()
        .expect("run string if-expression fixture");
    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "string if-expression fixture exited unexpectedly\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

fn compiler_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize compiler root")
}

fn std_link(compiler_root: &Path) -> PathBuf {
    let tmp_std_root = PathBuf::from("/tmp/vais-lib");
    std::fs::create_dir_all(&tmp_std_root).expect("failed to create /tmp/vais-lib");
    let std_link = tmp_std_root.join("std");
    let compiler_std = compiler_root.join("std");
    if std_link.exists() {
        let already_correct = std_link
            .canonicalize()
            .map(|path| path == compiler_std)
            .unwrap_or(false);
        if !already_correct {
            let _ = std::fs::remove_file(&std_link);
            let _ = std::fs::remove_dir(&std_link);
        }
    }
    if !std_link.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&compiler_std, &std_link)
            .expect("failed to create /tmp/vais-lib/std symlink");
    }
    std_link
}
