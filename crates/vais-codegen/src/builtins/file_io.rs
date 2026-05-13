use super::*;

impl CodeGenerator {
    pub(super) fn register_file_functions(&mut self) {
        // fopen: (path, mode) -> FILE*
        register_extern!(
            self,
            "fopen",
            vec![
                (String::from("path"), ResolvedType::Str),
                (String::from("mode"), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fopen_ptr: same as fopen but accepts i64 pointers (for selfhost)
        register_extern!(
            self,
            "fopen_ptr",
            vec![
                (String::from("path"), ResolvedType::I64),
                (String::from("mode"), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fclose: (FILE*) -> int
        register_extern!(
            self,
            "fclose",
            vec![(String::from("stream"), ResolvedType::I64)],
            ResolvedType::I32
        );

        // fread: (ptr, size, count, FILE*) -> size_t
        register_extern!(
            self,
            "fread",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("size"), ResolvedType::I64),
                (String::from("count"), ResolvedType::I64),
                (String::from("stream"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fwrite: (ptr, size, count, FILE*) -> size_t
        register_extern!(
            self,
            "fwrite",
            vec![
                (String::from("ptr"), ResolvedType::I64),
                (String::from("size"), ResolvedType::I64),
                (String::from("count"), ResolvedType::I64),
                (String::from("stream"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fgetc: (FILE*) -> int
        register_extern!(
            self,
            "fgetc",
            vec![(String::from("stream"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fputc: (char, FILE*) -> int
        register_extern!(
            self,
            "fputc",
            vec![
                (String::from("c"), ResolvedType::I64),
                (String::from("stream"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fgets_ptr: (str, i32, str) -> i64 - fgets with correct pointer types
        register_extern!(self, "fgets_ptr" => "fgets",
            vec![
                (String::from("buffer"), ResolvedType::Str),
                (String::from("n"), ResolvedType::I32),
                (String::from("stream"), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // fgets: (str, n, FILE*) -> char*
        register_extern!(
            self,
            "fgets",
            vec![
                (String::from("str"), ResolvedType::I64),
                (String::from("n"), ResolvedType::I64),
                (String::from("stream"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fputs: (str, FILE*) -> int
        register_extern!(
            self,
            "fputs",
            vec![
                (String::from("str"), ResolvedType::Str),
                (String::from("stream"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // fseek: (FILE*, offset, origin) -> int
        register_extern!(
            self,
            "fseek",
            vec![
                (String::from("stream"), ResolvedType::I64),
                (String::from("offset"), ResolvedType::I64),
                (String::from("origin"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // ftell: (FILE*) -> long
        register_extern!(
            self,
            "ftell",
            vec![(String::from("stream"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fflush: (FILE*) -> int
        register_extern!(
            self,
            "fflush",
            vec![(String::from("stream"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // feof: (FILE*) -> int
        register_extern!(
            self,
            "feof",
            vec![(String::from("stream"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fileno: (FILE*) -> int (get file descriptor from FILE*)
        register_extern!(
            self,
            "fileno",
            vec![(String::from("stream"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fsync: (fd) -> int (flush to disk)
        register_extern!(
            self,
            "fsync",
            vec![(String::from("fd"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // fdatasync: (fd) -> int (flush data only, no metadata)
        // On macOS, mapped to fcntl F_FULLFSYNC or fsync fallback
        register_extern!(
            self,
            "fdatasync",
            vec![(String::from("fd"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // mmap: (addr, len, prot, flags, fd, offset) -> void* (as i64)
        register_extern!(
            self,
            "mmap",
            vec![
                (String::from("addr"), ResolvedType::I64),
                (String::from("len"), ResolvedType::I64),
                (String::from("prot"), ResolvedType::I64),
                (String::from("flags"), ResolvedType::I64),
                (String::from("fd"), ResolvedType::I64),
                (String::from("offset"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // munmap: (addr, len) -> int
        register_extern!(
            self,
            "munmap",
            vec![
                (String::from("addr"), ResolvedType::I64),
                (String::from("len"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // msync: (addr, len, flags) -> int
        register_extern!(
            self,
            "msync",
            vec![
                (String::from("addr"), ResolvedType::I64),
                (String::from("len"), ResolvedType::I64),
                (String::from("flags"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // madvise: (addr, len, advice) -> int
        register_extern!(
            self,
            "madvise",
            vec![
                (String::from("addr"), ResolvedType::I64),
                (String::from("len"), ResolvedType::I64),
                (String::from("advice"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // POSIX open: (path, flags, mode) -> fd
        register_extern!(self, "posix_open" => "open",
            vec![
                (String::from("path"), ResolvedType::Str),
                (String::from("flags"), ResolvedType::I64),
                (String::from("mode"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // POSIX close: (fd) -> int
        register_extern!(self, "posix_close" => "close",
            vec![(String::from("fd"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // remove: (path) -> int (delete file)
        register_extern!(
            self,
            "remove",
            vec![(String::from("path"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // flock: (fd, operation) -> int (advisory file locking)
        register_extern!(
            self,
            "flock",
            vec![
                (String::from("fd"), ResolvedType::I64),
                (String::from("operation"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // mkdir: (path, mode) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "mkdir",
            vec![
                (String::from("path"), ResolvedType::Str),
                (String::from("mode"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // rmdir: (path) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "rmdir",
            vec![(String::from("path"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // opendir: (path) -> DIR* (as i64, 0 on error)
        register_extern!(
            self,
            "opendir",
            vec![(String::from("path"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // readdir: (dirp) -> dirent* (pointer to name, 0 at end)
        register_helper!(self, "readdir" => "__readdir_wrapper",
            vec![(String::from("dirp"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // closedir: (dirp) -> int (0 on success)
        register_extern!(
            self,
            "closedir",
            vec![(String::from("dirp"), ResolvedType::I64)],
            ResolvedType::I64
        );

        // rename_file: (old, new) -> int (0 on success) - maps to C rename()
        register_extern!(self, "rename_file" => "rename",
            vec![
                (String::from("old"), ResolvedType::Str),
                (String::from("new_path"), ResolvedType::Str),
            ],
            ResolvedType::I64
        );

        // unlink: (path) -> int (0 on success)
        register_extern!(
            self,
            "unlink",
            vec![(String::from("path"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // stat_size: (path) -> i64 (file size in bytes)
        register_helper!(self, "stat_size" => "__stat_size",
            vec![(String::from("path"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // stat_mtime: (path) -> i64 (modification time as unix timestamp)
        register_helper!(self, "stat_mtime" => "__stat_mtime",
            vec![(String::from("path"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // getcwd: (buf, size) -> i64 (pointer as i64, 0 on error) — needs wrapper for ptr→i64 conversion
        register_helper!(self, "getcwd" => "__getcwd_wrapper",
            vec![
                (String::from("buf"), ResolvedType::I64),
                (String::from("size"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );

        // chdir: (path) -> int (0 on success)
        register_extern!(
            self,
            "chdir",
            vec![(String::from("path"), ResolvedType::Str)],
            ResolvedType::I64
        );

        // access: (path, mode) -> int (0 on success, -1 on error)
        register_extern!(
            self,
            "access",
            vec![
                (String::from("path"), ResolvedType::Str),
                (String::from("mode"), ResolvedType::I64),
            ],
            ResolvedType::I64
        );
    }
}
