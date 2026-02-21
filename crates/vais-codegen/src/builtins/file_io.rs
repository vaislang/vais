use super::*;

impl CodeGenerator {
    pub(super) fn register_file_functions(&mut self) {
        // fopen: (path, mode) -> FILE*
        register_extern!(
            self,
            "fopen",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fopen_ptr: same as fopen but accepts i64 pointers (for selfhost)
        register_extern!(
            self,
            "fopen_ptr",
            vec![
                ("path".to_string(), ResolvedType::I64),
                ("mode".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fclose: (FILE*) -> int
        register_extern!(
            self,
            "fclose",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I32
        );

        // fread: (ptr, size, count, FILE*) -> size_t
        register_extern!(
            self,
            "fread",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("size".to_string(), ResolvedType::I64),
                ("count".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fwrite: (ptr, size, count, FILE*) -> size_t
        register_extern!(
            self,
            "fwrite",
            vec![
                ("ptr".to_string(), ResolvedType::I64),
                ("size".to_string(), ResolvedType::I64),
                ("count".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fgetc: (FILE*) -> int
        register_extern!(
            self,
            "fgetc",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fputc: (char, FILE*) -> int
        register_extern!(
            self,
            "fputc",
            vec![
                ("c".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fgets_ptr: (str, i32, str) -> i64 - fgets with correct pointer types
        register_extern!(self, "fgets_ptr" => "fgets",
            vec![
                ("buffer".to_string(), ResolvedType::Str),
                ("n".to_string(), ResolvedType::I32),
                ("stream".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fgets: (str, n, FILE*) -> char*
        register_extern!(
            self,
            "fgets",
            vec![
                ("str".to_string(), ResolvedType::I64),
                ("n".to_string(), ResolvedType::I64),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fputs: (str, FILE*) -> int
        register_extern!(
            self,
            "fputs",
            vec![
                ("str".to_string(), ResolvedType::Str),
                ("stream".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fseek: (FILE*, offset, origin) -> int
        register_extern!(
            self,
            "fseek",
            vec![
                ("stream".to_string(), ResolvedType::I64),
                ("offset".to_string(), ResolvedType::I64),
                ("origin".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // ftell: (FILE*) -> long
        register_extern!(
            self,
            "ftell",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fflush: (FILE*) -> int
        register_extern!(
            self,
            "fflush",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // feof: (FILE*) -> int
        register_extern!(
            self,
            "feof",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fileno: (FILE*) -> int (get file descriptor from FILE*)
        register_extern!(
            self,
            "fileno",
            vec![("stream".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fsync: (fd) -> int (flush to disk)
        register_extern!(
            self,
            "fsync",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fdatasync: (fd) -> int (flush data only, no metadata)
        // On macOS, mapped to fcntl F_FULLFSYNC or fsync fallback
        register_extern!(
            self,
            "fdatasync",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // mmap: (addr, len, prot, flags, fd, offset) -> void* (as i64)
        register_extern!(
            self,
            "mmap",
            vec![
                ("addr".to_string(), ResolvedType::I64),
                ("len".to_string(), ResolvedType::I64),
                ("prot".to_string(), ResolvedType::I64),
                ("flags".to_string(), ResolvedType::I64),
                ("fd".to_string(), ResolvedType::I64),
                ("offset".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // munmap: (addr, len) -> int
        register_extern!(
            self,
            "munmap",
            vec![
                ("addr".to_string(), ResolvedType::I64),
                ("len".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // msync: (addr, len, flags) -> int
        register_extern!(
            self,
            "msync",
            vec![
                ("addr".to_string(), ResolvedType::I64),
                ("len".to_string(), ResolvedType::I64),
                ("flags".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // madvise: (addr, len, advice) -> int
        register_extern!(
            self,
            "madvise",
            vec![
                ("addr".to_string(), ResolvedType::I64),
                ("len".to_string(), ResolvedType::I64),
                ("advice".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // POSIX open: (path, flags, mode) -> fd
        register_extern!(self, "posix_open" => "open",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("flags".to_string(), ResolvedType::I64),
                ("mode".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // POSIX close: (fd) -> int
        register_extern!(self, "posix_close" => "close",
            vec![("fd".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // remove: (path) -> int (delete file)
        register_extern!(
            self,
            "remove",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // flock: (fd, operation) -> int (advisory file locking)
        register_extern!(
            self,
            "flock",
            vec![
                ("fd".to_string(), ResolvedType::I64),
                ("operation".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // mkdir: (path, mode) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "mkdir",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // rmdir: (path) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "rmdir",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // opendir: (path) -> DIR* (as i64, 0 on error)
        register_extern!(
            self,
            "opendir",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // readdir: (dirp) -> dirent* (pointer to name, 0 at end)
        register_helper!(self, "readdir" => "__readdir_wrapper",
            vec![("dirp".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // closedir: (dirp) -> int (0 on success)
        register_extern!(
            self,
            "closedir",
            vec![("dirp".to_string(), ResolvedType::I64)],
            ResolvedType::I64
        );

        // rename_file: (old, new) -> int (0 on success) - maps to C rename()
        register_extern!(self, "rename_file" => "rename",
            vec![
                ("old".to_string(), ResolvedType::Str),
                ("new_path".to_string(), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // unlink: (path) -> int (0 on success)
        register_extern!(
            self,
            "unlink",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // stat_size: (path) -> i64 (file size in bytes)
        register_helper!(self, "stat_size" => "__stat_size",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // stat_mtime: (path) -> i64 (modification time as unix timestamp)
        register_helper!(self, "stat_mtime" => "__stat_mtime",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // getcwd: (buf, size) -> i64 (pointer as i64, 0 on error) — needs wrapper for ptr→i64 conversion
        register_helper!(self, "getcwd" => "__getcwd_wrapper",
            vec![
                ("buf".to_string(), ResolvedType::I64),
                ("size".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // chdir: (path) -> int (0 on success)
        register_extern!(
            self,
            "chdir",
            vec![("path".to_string(), ResolvedType::Str)],
            ResolvedType::I64
        );

        // access: (path, mode) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "access",
            vec![
                ("path".to_string(), ResolvedType::Str),
                ("mode".to_string(), ResolvedType::I64),
            ],
            ResolvedType::I64
        );
    }
}
