//! File I/O and filesystem built-in functions

use super::*;

impl TypeChecker {
    pub(super) fn register_file_io_builtins(&mut self) {
        // ===== File I/O functions =====

        // fopen: (path, mode) -> FILE* (as i64)
        self.functions.insert(
            "fopen".to_string(),
            FunctionSig {
                name: "fopen".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fopen_ptr: same as fopen but accepts i64 pointer (for selfhost)
        self.functions.insert(
            "fopen_ptr".to_string(),
            FunctionSig {
                name: "fopen_ptr".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::I64, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fclose: (stream) -> i32
        self.functions.insert(
            "fclose".to_string(),
            FunctionSig {
                name: "fclose".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // fread: (ptr, size, count, stream) -> i64
        self.functions.insert(
            "fread".to_string(),
            FunctionSig {
                name: "fread".to_string(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("count".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fwrite: (ptr, size, count, stream) -> i64
        self.functions.insert(
            "fwrite".to_string(),
            FunctionSig {
                name: "fwrite".to_string(),
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("count".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fgetc: (stream) -> i64 (returns -1 on EOF)
        self.functions.insert(
            "fgetc".to_string(),
            FunctionSig {
                name: "fgetc".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fputc: (c, stream) -> i64
        self.functions.insert(
            "fputc".to_string(),
            FunctionSig {
                name: "fputc".to_string(),
                params: vec![
                    ("c".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fgets: (str, n, stream) -> i64 (char*)
        self.functions.insert(
            "fgets".to_string(),
            FunctionSig {
                name: "fgets".to_string(),
                params: vec![
                    ("str".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fputs: (str, stream) -> i64
        self.functions.insert(
            "fputs".to_string(),
            FunctionSig {
                name: "fputs".to_string(),
                params: vec![
                    ("str".to_string(), ResolvedType::Str, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fseek: (stream, offset, origin) -> i64
        self.functions.insert(
            "fseek".to_string(),
            FunctionSig {
                name: "fseek".to_string(),
                params: vec![
                    ("stream".to_string(), ResolvedType::I64, false),
                    ("offset".to_string(), ResolvedType::I64, false),
                    ("origin".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // ftell: (stream) -> i64
        self.functions.insert(
            "ftell".to_string(),
            FunctionSig {
                name: "ftell".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fflush: (stream) -> i64
        self.functions.insert(
            "fflush".to_string(),
            FunctionSig {
                name: "fflush".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // feof: (stream) -> i64
        self.functions.insert(
            "feof".to_string(),
            FunctionSig {
                name: "feof".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fileno: (stream) -> i64
        self.functions.insert(
            "fileno".to_string(),
            FunctionSig {
                name: "fileno".to_string(),
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fsync: (fd) -> i64
        self.functions.insert(
            "fsync".to_string(),
            FunctionSig {
                name: "fsync".to_string(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // fdatasync: (fd) -> i64
        self.functions.insert(
            "fdatasync".to_string(),
            FunctionSig {
                name: "fdatasync".to_string(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // mmap: (addr, len, prot, flags, fd, offset) -> ptr
        self.functions.insert(
            "mmap".to_string(),
            FunctionSig {
                name: "mmap".to_string(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("prot".to_string(), ResolvedType::I64, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                    ("fd".to_string(), ResolvedType::I64, false),
                    ("offset".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // munmap: (addr, len) -> int
        self.functions.insert(
            "munmap".to_string(),
            FunctionSig {
                name: "munmap".to_string(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // msync: (addr, len, flags) -> int
        self.functions.insert(
            "msync".to_string(),
            FunctionSig {
                name: "msync".to_string(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // madvise: (addr, len, advice) -> int
        self.functions.insert(
            "madvise".to_string(),
            FunctionSig {
                name: "madvise".to_string(),
                params: vec![
                    ("addr".to_string(), ResolvedType::I64, false),
                    ("len".to_string(), ResolvedType::I64, false),
                    ("advice".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // posix_open: (path, flags, mode) -> fd
        self.functions.insert(
            "posix_open".to_string(),
            FunctionSig {
                name: "posix_open".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // posix_close: (fd) -> i64
        self.functions.insert(
            "posix_close".to_string(),
            FunctionSig {
                name: "posix_close".to_string(),
                params: vec![("fd".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // remove: (path) -> i64
        self.functions.insert(
            "remove".to_string(),
            FunctionSig {
                name: "remove".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // flock: (fd, operation) -> i64 (advisory file locking)
        self.functions.insert(
            "flock".to_string(),
            FunctionSig {
                name: "flock".to_string(),
                params: vec![
                    ("fd".to_string(), ResolvedType::I64, false),
                    ("operation".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // mkdir: (path, mode) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "mkdir".to_string(),
            FunctionSig {
                name: "mkdir".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // rmdir: (path) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "rmdir".to_string(),
            FunctionSig {
                name: "rmdir".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // opendir: (path) -> i64 (DIR* as i64, 0 on error)
        self.functions.insert(
            "opendir".to_string(),
            FunctionSig {
                name: "opendir".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // readdir: (dirp) -> i64 (pointer to dirent name, 0 at end)
        self.functions.insert(
            "readdir".to_string(),
            FunctionSig {
                name: "readdir".to_string(),
                params: vec![("dirp".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // closedir: (dirp) -> i64 (0 on success)
        self.functions.insert(
            "closedir".to_string(),
            FunctionSig {
                name: "closedir".to_string(),
                params: vec![("dirp".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // rename_file: (old, new_path) -> i64 (0 on success)
        self.functions.insert(
            "rename_file".to_string(),
            FunctionSig {
                name: "rename_file".to_string(),
                params: vec![
                    ("old".to_string(), ResolvedType::Str, false),
                    ("new_path".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // unlink: (path) -> i64 (0 on success)
        self.functions.insert(
            "unlink".to_string(),
            FunctionSig {
                name: "unlink".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // stat_size: (path) -> i64 (file size in bytes)
        self.functions.insert(
            "stat_size".to_string(),
            FunctionSig {
                name: "stat_size".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // stat_mtime: (path) -> i64 (modification time as unix timestamp)
        self.functions.insert(
            "stat_mtime".to_string(),
            FunctionSig {
                name: "stat_mtime".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // getcwd: (buf, size) -> i64 (pointer to buf on success, 0 on error)
        self.functions.insert(
            "getcwd".to_string(),
            FunctionSig {
                name: "getcwd".to_string(),
                params: vec![
                    ("buf".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // chdir: (path) -> i64 (0 on success)
        self.functions.insert(
            "chdir".to_string(),
            FunctionSig {
                name: "chdir".to_string(),
                params: vec![("path".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // access: (path, mode) -> i64 (0 on success, -1 on error)
        self.functions.insert(
            "access".to_string(),
            FunctionSig {
                name: "access".to_string(),
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );
    }
}
