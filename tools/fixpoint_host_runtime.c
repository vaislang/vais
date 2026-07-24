#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

extern int64_t vais_user_main(void);

static int64_t vais_argc = 0;
static char **vais_argv = NULL;

void vais_list_trap(int64_t kind) {
    if (kind == 3) fprintf(stderr, "vais list trap: capacity exceeded (fixed list contract: 4095 entries)\n");
    else if (kind == 2) fprintf(stderr, "vais list trap: empty-list access\n");
    else fprintf(stderr, "vais list trap: index out of range\n");
    abort();
}

static void host_trap(const char *name) {
    fprintf(stderr, "vais host runtime trap: %s\n", name);
    abort();
}

static char *copy_n(const char *text, size_t len) {
    char *out = (char *)malloc(len + 1);
    if (out == NULL) host_trap("alloc");
    if (len > 0) memcpy(out, text, len);
    out[len] = '\0';
    return out;
}

static char *copy_str(const char *text) {
    if (text == NULL) return copy_n("", 0);
    return copy_n(text, strlen(text));
}

int64_t time_millis(void) {
#if defined(CLOCK_MONOTONIC)
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) == 0) {
        return (int64_t)ts.tv_sec * 1000 + (int64_t)(ts.tv_nsec / 1000000);
    }
#endif
    struct timeval tv;
    if (gettimeofday(&tv, NULL) != 0) return 0;
    return (int64_t)tv.tv_sec * 1000 + (int64_t)(tv.tv_usec / 1000);
}

static const char *vais_self = "";

char *proc_self(void) {
    return copy_str(vais_self);
}

int main(int argc, char **argv) {
    vais_self = argc > 0 ? argv[0] : "";
    vais_argc = argc > 0 ? (int64_t)argc - 1 : 0;
    vais_argv = argc > 1 ? argv + 1 : NULL;
    return (int)vais_user_main();
}

int64_t fs_exists(char *path) {
    if (path == NULL) return 0;
    return access(path, F_OK) == 0 ? 1 : 0;
}

char *fs_read_text(char *path) {
    if (path == NULL) host_trap("fs_read_text");
    FILE *fp = fopen(path, "rb");
    if (fp == NULL) host_trap("fs_read_text");
    if (fseek(fp, 0, SEEK_END) != 0) host_trap("fs_read_text");
    long len = ftell(fp);
    if (len < 0) host_trap("fs_read_text");
    if (fseek(fp, 0, SEEK_SET) != 0) host_trap("fs_read_text");
    char *out = (char *)malloc((size_t)len + 1);
    if (out == NULL) host_trap("fs_read_text");
    size_t got = fread(out, 1, (size_t)len, fp);
    if (got != (size_t)len && ferror(fp)) host_trap("fs_read_text");
    fclose(fp);
    out[got] = '\0';
    return out;
}

int64_t fs_write_text(char *path, char *text) {
    if (path == NULL || text == NULL) return 1;
    FILE *fp = fopen(path, "wb");
    if (fp == NULL) return errno == 0 ? 1 : errno;
    if (fputs(text, fp) < 0) {
        int err = errno == 0 ? 1 : errno;
        fclose(fp);
        return err;
    }
    if (fclose(fp) != 0) return errno == 0 ? 1 : errno;
    return 0;
}

char *fs_cwd(void) {
    char buf[4096];
    if (getcwd(buf, sizeof(buf)) == NULL) host_trap("fs_cwd");
    return copy_str(buf);
}

char *fs_temp_dir(void) {
    const char *tmp = getenv("TMPDIR");
    if (tmp == NULL || tmp[0] == '\0') tmp = "/tmp";
    return copy_str(tmp);
}

static int64_t mkdir_one(const char *path) {
    if (path == NULL || path[0] == '\0') return 0;
    if (mkdir(path, 0777) == 0) return 0;
    if (errno == EEXIST) {
        struct stat st;
        if (stat(path, &st) == 0 && S_ISDIR(st.st_mode)) return 0;
        return ENOTDIR;
    }
    return errno == 0 ? 1 : errno;
}

int64_t fs_mkdirs(char *path) {
    if (path == NULL || path[0] == '\0') return 1;
    char buf[4096];
    size_t len = strlen(path);
    if (len >= sizeof(buf)) return ENAMETOOLONG;
    memcpy(buf, path, len + 1);
    while (len > 1 && buf[len - 1] == '/') {
        buf[len - 1] = '\0';
        len--;
    }
    for (char *p = buf + 1; *p != '\0'; p++) {
        if (*p == '/') {
            *p = '\0';
            int64_t rc = mkdir_one(buf);
            *p = '/';
            if (rc != 0) return rc;
        }
    }
    return mkdir_one(buf);
}

int64_t fs_remove(char *path) {
    if (path == NULL || path[0] == '\0') return 1;
    if (unlink(path) == 0) return 0;
    if (errno == ENOENT) return 0;
    return errno == 0 ? 1 : errno;
}

char *path_join(char *base, char *child) {
    if (child == NULL) host_trap("path_join");
    if (child[0] == '/') return copy_str(child);
    if (base == NULL || base[0] == '\0') return copy_str(child);
    size_t a = strlen(base);
    size_t b = strlen(child);
    int need_slash = base[a - 1] != '/';
    char *out = (char *)malloc(a + (size_t)need_slash + b + 1);
    if (out == NULL) host_trap("path_join");
    memcpy(out, base, a);
    size_t pos = a;
    if (need_slash) out[pos++] = '/';
    memcpy(out + pos, child, b);
    out[pos + b] = '\0';
    return out;
}

char *path_basename(char *path) {
    if (path == NULL || path[0] == '\0') return copy_str("");
    char *last = strrchr(path, '/');
    return copy_str(last == NULL ? path : last + 1);
}

char *path_dirname(char *path) {
    if (path == NULL || path[0] == '\0') return copy_str(".");
    char *last = strrchr(path, '/');
    if (last == NULL) return copy_str(".");
    if (last == path) return copy_str("/");
    return copy_n(path, (size_t)(last - path));
}

char *str_concat(char *left, char *right) {
    if (left == NULL || right == NULL) host_trap("str_concat");
    size_t a = strlen(left);
    size_t b = strlen(right);
    char *out = (char *)malloc(a + b + 1);
    if (out == NULL) host_trap("str_concat");
    memcpy(out, left, a);
    memcpy(out + a, right, b);
    out[a + b] = '\0';
    return out;
}

char *str_slice(char *text, int64_t start, int64_t len) {
    if (text == NULL || start < 0 || len < 0) host_trap("str_slice");
    size_t n = strlen(text);
    if ((size_t)start > n || (size_t)len > n - (size_t)start) host_trap("str_slice");
    return copy_n(text + start, (size_t)len);
}

char *str_byte(int64_t value) {
    if (value < 0 || value > 255) host_trap("str_byte");
    char *out = (char *)malloc(2);
    if (out == NULL) host_trap("str_byte");
    out[0] = (char)value;
    out[1] = '\0';
    return out;
}

typedef struct {
    char *data;
    size_t len;
    size_t cap;
} Builder;

static Builder *builder_from_handle(int64_t handle) {
    Builder *b = (Builder *)(intptr_t)handle;
    if (b == NULL) host_trap("str_builder");
    return b;
}

static void builder_reserve(Builder *b, size_t extra) {
    size_t need = b->len + extra + 1;
    if (need <= b->cap) return;
    size_t cap = b->cap == 0 ? 64 : b->cap;
    while (cap < need) cap *= 2;
    char *next = (char *)realloc(b->data, cap);
    if (next == NULL) host_trap("str_builder");
    b->data = next;
    b->cap = cap;
}

int64_t str_builder_new(void) {
    Builder *b = (Builder *)calloc(1, sizeof(Builder));
    if (b == NULL) host_trap("str_builder_new");
    builder_reserve(b, 0);
    b->data[0] = '\0';
    return (int64_t)(intptr_t)b;
}

int64_t str_builder_push(int64_t handle, int64_t value) {
    if (value < 0 || value > 255) host_trap("str_builder_push");
    Builder *b = builder_from_handle(handle);
    builder_reserve(b, 1);
    b->data[b->len++] = (char)value;
    b->data[b->len] = '\0';
    return 0;
}

int64_t str_builder_append(int64_t handle, char *text) {
    if (text == NULL) host_trap("str_builder_append");
    Builder *b = builder_from_handle(handle);
    size_t len = strlen(text);
    builder_reserve(b, len);
    memcpy(b->data + b->len, text, len);
    b->len += len;
    b->data[b->len] = '\0';
    return 0;
}

char *str_builder_finish(int64_t handle) {
    Builder *b = builder_from_handle(handle);
    return copy_n(b->data, b->len);
}

int64_t proc_argc(void) {
    return vais_argc;
}

char *proc_arg(int64_t index) {
    if (index < 0 || index >= vais_argc || vais_argv == NULL) return copy_str("");
    return copy_str(vais_argv[index]);
}

static char **argv_from_list(int64_t *argv_buf, int64_t *argc_out) {
    if (argv_buf == NULL) return NULL;
    int64_t argc = argv_buf[4095];
    if (argc <= 0 || argc > 4095) return NULL;
    char **argv = (char **)malloc((size_t)(argc + 1) * sizeof(char *));
    if (argv == NULL) return NULL;
    for (int64_t i = 0; i < argc; i++) {
        argv[i] = (char *)(intptr_t)argv_buf[i];
        if (argv[i] == NULL) {
            free(argv);
            return NULL;
        }
    }
    argv[argc] = NULL;
    *argc_out = argc;
    return argv;
}

static int redirect_path(char *path, int fd) {
    if (path == NULL || path[0] == '\0') return 0;
    FILE *fp = fopen(path, "wb");
    if (fp == NULL) return 1;
    int target = fileno(fp);
    if (dup2(target, fd) < 0) {
        fclose(fp);
        return 1;
    }
    fclose(fp);
    return 0;
}

int64_t proc_capture_to(int64_t *argv_buf, char *stdout_path, char *stderr_path) {
    int64_t argc = 0;
    char **argv = argv_from_list(argv_buf, &argc);
    if (argv == NULL) return 1;
    pid_t pid = fork();
    if (pid < 0) {
        free(argv);
        return errno == 0 ? 1 : errno;
    }
    if (pid == 0) {
        if (redirect_path(stdout_path, STDOUT_FILENO) != 0) _exit(127);
        if (redirect_path(stderr_path, STDERR_FILENO) != 0) _exit(127);
        execvp(argv[0], argv);
        _exit(127);
    }
    int status = 0;
    while (waitpid(pid, &status, 0) < 0) {
        if (errno == EINTR) continue;
        free(argv);
        return errno == 0 ? 1 : errno;
    }
    free(argv);
    if (WIFEXITED(status)) return WEXITSTATUS(status);
    if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);
    return 1;
}

int64_t proc_run(int64_t *argv_buf) {
    return proc_capture_to(argv_buf, "", "");
}

int64_t proc_run_env(int64_t *argv_buf, int64_t *env_buf) {
    (void)env_buf;
    return proc_run(argv_buf);
}

char *proc_capture_stdout(int64_t *argv_buf) {
    char out_template[] = "/tmp/vais-fixpoint-out-XXXXXX";
    char err_template[] = "/tmp/vais-fixpoint-err-XXXXXX";
    int out_fd = mkstemp(out_template);
    int err_fd = mkstemp(err_template);
    if (out_fd >= 0) close(out_fd);
    if (err_fd >= 0) close(err_fd);
    if (out_fd < 0 || err_fd < 0) return copy_str("");
    proc_capture_to(argv_buf, out_template, err_template);
    char *out = fs_read_text(out_template);
    unlink(out_template);
    unlink(err_template);
    return out;
}

char *proc_capture_stderr(int64_t *argv_buf) {
    char out_template[] = "/tmp/vais-fixpoint-out-XXXXXX";
    char err_template[] = "/tmp/vais-fixpoint-err-XXXXXX";
    int out_fd = mkstemp(out_template);
    int err_fd = mkstemp(err_template);
    if (out_fd >= 0) close(out_fd);
    if (err_fd >= 0) close(err_fd);
    if (out_fd < 0 || err_fd < 0) return copy_str("");
    proc_capture_to(argv_buf, out_template, err_template);
    char *err = fs_read_text(err_template);
    unlink(out_template);
    unlink(err_template);
    return err;
}

void proc_capture(int64_t *argv_buf, int64_t *out) {
    if (out == NULL) return;
    char out_template[] = "/tmp/vais-fixpoint-out-XXXXXX";
    char err_template[] = "/tmp/vais-fixpoint-err-XXXXXX";
    int out_fd = mkstemp(out_template);
    int err_fd = mkstemp(err_template);
    if (out_fd >= 0) close(out_fd);
    if (err_fd >= 0) close(err_fd);
    if (out_fd < 0 || err_fd < 0) {
        out[0] = 1;
        out[1] = (int64_t)(intptr_t)copy_str("");
        out[2] = (int64_t)(intptr_t)copy_str("");
        return;
    }
    int64_t code = proc_capture_to(argv_buf, out_template, err_template);
    out[0] = code;
    out[1] = (int64_t)(intptr_t)fs_read_text(out_template);
    out[2] = (int64_t)(intptr_t)fs_read_text(err_template);
    unlink(out_template);
    unlink(err_template);
}
