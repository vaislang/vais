// Thread runtime support for VAIS
// Provides pthread-based threading, TLS, and utility functions
//
// Linked automatically by vaisc when std/thread is imported.

#define _GNU_SOURCE  // Required for pthread_tryjoin_np on Linux
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>
#include <time.h>
#include <sched.h>

// ============================================
// Thread entry point wrapper
// ============================================

typedef struct {
    long (*fn_ptr)(long);
    long arg;
    long result_ptr;
} thread_args_t;

typedef struct {
    long (*closure_ptr)(long);
    long env_ptr;
    long result_ptr;
} closure_args_t;

static void* thread_entry(void* raw) {
    thread_args_t* args = (thread_args_t*)raw;
    long result = args->fn_ptr(args->arg);
    if (args->result_ptr) {
        *(long*)args->result_ptr = result;
    }
    free(args);
    return NULL;
}

static void* closure_entry(void* raw) {
    closure_args_t* args = (closure_args_t*)raw;
    long result = args->closure_ptr(args->env_ptr);
    if (args->result_ptr) {
        *(long*)args->result_ptr = result;
    }
    free(args);
    return NULL;
}

// ============================================
// Thread spawn/join
// ============================================

long __thread_spawn(long fn_ptr, long arg, long result_ptr) {
    thread_args_t* args = (thread_args_t*)malloc(sizeof(thread_args_t));
    if (!args) return 0;
    args->fn_ptr = (long (*)(long))fn_ptr;
    args->arg = arg;
    args->result_ptr = result_ptr;

    pthread_t* thread = (pthread_t*)malloc(sizeof(pthread_t));
    if (!thread) { free(args); return 0; }

    if (pthread_create(thread, NULL, thread_entry, args) != 0) {
        free(args);
        free(thread);
        return 0;
    }
    return (long)thread;
}

long __thread_spawn_closure(long closure_ptr, long env_ptr, long result_ptr) {
    closure_args_t* args = (closure_args_t*)malloc(sizeof(closure_args_t));
    if (!args) return 0;
    args->closure_ptr = (long (*)(long))closure_ptr;
    args->env_ptr = env_ptr;
    args->result_ptr = result_ptr;

    pthread_t* thread = (pthread_t*)malloc(sizeof(pthread_t));
    if (!thread) { free(args); return 0; }

    if (pthread_create(thread, NULL, closure_entry, args) != 0) {
        free(args);
        free(thread);
        return 0;
    }
    return (long)thread;
}

long __thread_spawn_with_options(long fn_ptr, long arg, long result_ptr, long stack_size) {
    thread_args_t* args = (thread_args_t*)malloc(sizeof(thread_args_t));
    if (!args) return 0;
    args->fn_ptr = (long (*)(long))fn_ptr;
    args->arg = arg;
    args->result_ptr = result_ptr;

    pthread_attr_t attr;
    pthread_attr_init(&attr);
    if (stack_size > 0) {
        pthread_attr_setstacksize(&attr, (size_t)stack_size);
    }

    pthread_t* thread = (pthread_t*)malloc(sizeof(pthread_t));
    if (!thread) { free(args); pthread_attr_destroy(&attr); return 0; }

    int rc = pthread_create(thread, &attr, thread_entry, args);
    pthread_attr_destroy(&attr);

    if (rc != 0) {
        free(args);
        free(thread);
        return 0;
    }
    return (long)thread;
}

long __thread_join(long handle) {
    if (!handle) return -1;
    pthread_t* thread = (pthread_t*)handle;
    int rc = pthread_join(*thread, NULL);
    free(thread);
    return (long)rc;
}

long __thread_try_join(long handle) {
    // Non-blocking check: use pthread_tryjoin_np on Linux,
    // fall back to simple check on other platforms
#ifdef __linux__
    if (!handle) return 0;
    pthread_t* thread = (pthread_t*)handle;
    int rc = pthread_tryjoin_np(*thread, NULL);
    if (rc == 0) {
        free(thread);
        return 1; // finished
    }
    return 0; // still running
#else
    // On macOS/other: no non-blocking join, return 0 (not finished)
    (void)handle;
    return 0;
#endif
}

long __thread_detach(long handle) {
    if (!handle) return -1;
    pthread_t* thread = (pthread_t*)handle;
    int rc = pthread_detach(*thread);
    free(thread);
    return (long)rc;
}

// ============================================
// Current thread info
// ============================================

long __thread_current(void) {
    // Return the pthread_t as an opaque handle
    // Note: pthread_self() returns by value, we store it
    pthread_t self = pthread_self();
    pthread_t* handle = (pthread_t*)malloc(sizeof(pthread_t));
    if (!handle) return 0;
    *handle = self;
    return (long)handle;
}

long __thread_current_id(void) {
    // Use a hash of pthread_self() as a portable thread ID
    pthread_t self = pthread_self();
    return (long)(size_t)self;
}

// ============================================
// Yield / Sleep / Park
// ============================================

long __thread_yield(void) {
    sched_yield();
    return 0;
}

long __thread_sleep_ms(long ms) {
    if (ms <= 0) return 0;
    struct timespec ts;
    ts.tv_sec = ms / 1000;
    ts.tv_nsec = (ms % 1000) * 1000000L;
    nanosleep(&ts, NULL);
    return 0;
}

// Park/unpark implementation using per-thread mutex+condvar
// Simplified: park sleeps for a short interval, unpark is a no-op
// A full implementation would require a per-thread park state table.

long __thread_park(void) {
    // Simple implementation: sleep 1ms in a loop
    // Real implementation would use futex/condvar
    struct timespec ts = {0, 1000000L}; // 1ms
    nanosleep(&ts, NULL);
    return 0;
}

long __thread_park_timeout(long ms) {
    if (ms <= 0) return 0;
    struct timespec ts;
    ts.tv_sec = ms / 1000;
    ts.tv_nsec = (ms % 1000) * 1000000L;
    nanosleep(&ts, NULL);
    return 0;
}

long __thread_unpark(long handle) {
    // Simplified: no-op (full impl would signal condvar)
    (void)handle;
    return 0;
}

// ============================================
// Thread-Local Storage
// ============================================

long __tls_create(void) {
    pthread_key_t* key = (pthread_key_t*)malloc(sizeof(pthread_key_t));
    if (!key) return 0;
    if (pthread_key_create(key, NULL) != 0) {
        free(key);
        return 0;
    }
    return (long)key;
}

long __tls_get(long key_handle) {
    if (!key_handle) return 0;
    pthread_key_t* key = (pthread_key_t*)key_handle;
    return (long)pthread_getspecific(*key);
}

long __tls_set(long key_handle, long value) {
    if (!key_handle) return -1;
    pthread_key_t* key = (pthread_key_t*)key_handle;
    return (long)pthread_setspecific(*key, (void*)value);
}

// ============================================
// Hardware info
// ============================================

long __cpu_count(void) {
#ifdef _SC_NPROCESSORS_ONLN
    long count = sysconf(_SC_NPROCESSORS_ONLN);
    return count > 0 ? count : 1;
#else
    return 1;
#endif
}

// ============================================
// Function call helpers
// ============================================

long __load_result(long ptr) {
    if (!ptr) return 0;
    return *(long*)ptr;
}

long __call_fn(long fn_ptr) {
    if (!fn_ptr) return 0;
    long (*f)(void) = (long (*)(void))fn_ptr;
    return f();
}

long __call_fn_with_arg(long fn_ptr, long arg) {
    if (!fn_ptr) return 0;
    long (*f)(long) = (long (*)(long))fn_ptr;
    return f(arg);
}

long __call_scope_fn(long fn_ptr, long scope_ptr) {
    if (!fn_ptr) return 0;
    long (*f)(long) = (long (*)(long))fn_ptr;
    return f(scope_ptr);
}
