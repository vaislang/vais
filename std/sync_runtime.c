// Sync runtime support for VAIS
// Provides pthread-based Mutex, RwLock, Condvar, Barrier, Once, Semaphore, and Atomics
//
// Linked automatically by vaisc when std/sync is imported.

#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdint.h>
#include <sched.h>

// ============================================
// Mutex
// ============================================

long __mutex_create(void) {
    pthread_mutex_t *m = (pthread_mutex_t *)malloc(sizeof(pthread_mutex_t));
    if (!m) return 0;
    pthread_mutex_init(m, NULL);
    return (long)m;
}

long __mutex_destroy(long handle) {
    if (!handle) return -1;
    pthread_mutex_t *m = (pthread_mutex_t *)handle;
    pthread_mutex_destroy(m);
    free(m);
    return 0;
}

long __mutex_lock(long handle) {
    if (!handle) return -1;
    return (long)pthread_mutex_lock((pthread_mutex_t *)handle);
}

long __mutex_try_lock(long handle) {
    if (!handle) return 0;
    int rc = pthread_mutex_trylock((pthread_mutex_t *)handle);
    return rc == 0 ? 1 : 0;
}

long __mutex_unlock(long handle) {
    if (!handle) return -1;
    return (long)pthread_mutex_unlock((pthread_mutex_t *)handle);
}

// ============================================
// RwLock
// ============================================

long __rwlock_create(void) {
    pthread_rwlock_t *rw = (pthread_rwlock_t *)malloc(sizeof(pthread_rwlock_t));
    if (!rw) return 0;
    pthread_rwlock_init(rw, NULL);
    return (long)rw;
}

long __rwlock_destroy(long handle) {
    if (!handle) return -1;
    pthread_rwlock_t *rw = (pthread_rwlock_t *)handle;
    pthread_rwlock_destroy(rw);
    free(rw);
    return 0;
}

long __rwlock_read_lock(long handle) {
    if (!handle) return -1;
    return (long)pthread_rwlock_rdlock((pthread_rwlock_t *)handle);
}

long __rwlock_try_read_lock(long handle) {
    if (!handle) return 0;
    int rc = pthread_rwlock_tryrdlock((pthread_rwlock_t *)handle);
    return rc == 0 ? 1 : 0;
}

long __rwlock_read_unlock(long handle) {
    if (!handle) return -1;
    return (long)pthread_rwlock_unlock((pthread_rwlock_t *)handle);
}

long __rwlock_write_lock(long handle) {
    if (!handle) return -1;
    return (long)pthread_rwlock_wrlock((pthread_rwlock_t *)handle);
}

long __rwlock_try_write_lock(long handle) {
    if (!handle) return 0;
    int rc = pthread_rwlock_trywrlock((pthread_rwlock_t *)handle);
    return rc == 0 ? 1 : 0;
}

long __rwlock_write_unlock(long handle) {
    if (!handle) return -1;
    return (long)pthread_rwlock_unlock((pthread_rwlock_t *)handle);
}

// ============================================
// Condvar
// ============================================

long __condvar_create(void) {
    pthread_cond_t *c = (pthread_cond_t *)malloc(sizeof(pthread_cond_t));
    if (!c) return 0;
    pthread_cond_init(c, NULL);
    return (long)c;
}

long __condvar_destroy(long handle) {
    if (!handle) return -1;
    pthread_cond_t *c = (pthread_cond_t *)handle;
    pthread_cond_destroy(c);
    free(c);
    return 0;
}

long __condvar_wait(long condvar, long mutex) {
    if (!condvar || !mutex) return -1;
    return (long)pthread_cond_wait(
        (pthread_cond_t *)condvar,
        (pthread_mutex_t *)mutex
    );
}

long __condvar_wait_timeout(long condvar, long mutex, long timeout_ms) {
    if (!condvar || !mutex) return -1;

    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    ts.tv_sec += timeout_ms / 1000;
    ts.tv_nsec += (timeout_ms % 1000) * 1000000L;
    if (ts.tv_nsec >= 1000000000L) {
        ts.tv_sec += 1;
        ts.tv_nsec -= 1000000000L;
    }

    int rc = pthread_cond_timedwait(
        (pthread_cond_t *)condvar,
        (pthread_mutex_t *)mutex,
        &ts
    );
    // Return 0 on success (signalled), 1 on timeout
    return rc == 0 ? 0 : 1;
}

long __condvar_signal(long handle) {
    if (!handle) return -1;
    return (long)pthread_cond_signal((pthread_cond_t *)handle);
}

long __condvar_broadcast(long handle) {
    if (!handle) return -1;
    return (long)pthread_cond_broadcast((pthread_cond_t *)handle);
}

// ============================================
// Barrier
// ============================================

// macOS does not have pthread_barrier_t, so we implement one
typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    int threshold;
    int count;
    int generation;
} vais_barrier_t;

long __barrier_create(long count) {
    if (count <= 0) return 0;
    vais_barrier_t *b = (vais_barrier_t *)malloc(sizeof(vais_barrier_t));
    if (!b) return 0;
    pthread_mutex_init(&b->mutex, NULL);
    pthread_cond_init(&b->cond, NULL);
    b->threshold = (int)count;
    b->count = 0;
    b->generation = 0;
    return (long)b;
}

long __barrier_destroy(long handle) {
    if (!handle) return -1;
    vais_barrier_t *b = (vais_barrier_t *)handle;
    pthread_mutex_destroy(&b->mutex);
    pthread_cond_destroy(&b->cond);
    free(b);
    return 0;
}

long __barrier_wait(long handle) {
    if (!handle) return -1;
    vais_barrier_t *b = (vais_barrier_t *)handle;

    pthread_mutex_lock(&b->mutex);
    int gen = b->generation;
    b->count++;

    if (b->count >= b->threshold) {
        // Last thread to arrive: reset and wake everyone
        b->count = 0;
        b->generation++;
        pthread_cond_broadcast(&b->cond);
        pthread_mutex_unlock(&b->mutex);
        return 1; // Leader
    }

    // Wait until generation changes
    while (gen == b->generation) {
        pthread_cond_wait(&b->cond, &b->mutex);
    }
    pthread_mutex_unlock(&b->mutex);
    return 0; // Non-leader
}

// ============================================
// Once
// ============================================

typedef struct {
    pthread_once_t once;
    long fn_ptr;
    int completed;
    pthread_mutex_t mutex;
} vais_once_t;

// Thread-local for passing fn_ptr to pthread_once handler
static __thread long _once_fn_ptr = 0;

static void once_handler(void) {
    if (_once_fn_ptr) {
        long (*fn)(void) = (long (*)(void))_once_fn_ptr;
        fn();
    }
}

long __once_create(void) {
    vais_once_t *o = (vais_once_t *)malloc(sizeof(vais_once_t));
    if (!o) return 0;
    // Cannot use PTHREAD_ONCE_INIT for dynamically allocated, use mutex-based approach
    pthread_mutex_init(&o->mutex, NULL);
    o->completed = 0;
    o->fn_ptr = 0;
    return (long)o;
}

long __once_call(long handle, long fn_ptr) {
    if (!handle || !fn_ptr) return -1;
    vais_once_t *o = (vais_once_t *)handle;

    pthread_mutex_lock(&o->mutex);
    if (!o->completed) {
        o->completed = 1;
        pthread_mutex_unlock(&o->mutex);
        // Call the function outside the lock to avoid deadlock
        long (*fn)(void) = (long (*)(void))fn_ptr;
        fn();
        return 0;
    }
    pthread_mutex_unlock(&o->mutex);
    return 0;
}

// ============================================
// Semaphore (portable implementation using mutex+condvar)
// ============================================

typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    long permits;
} vais_semaphore_t;

long __semaphore_create(long permits) {
    vais_semaphore_t *s = (vais_semaphore_t *)malloc(sizeof(vais_semaphore_t));
    if (!s) return 0;
    pthread_mutex_init(&s->mutex, NULL);
    pthread_cond_init(&s->cond, NULL);
    s->permits = permits;
    return (long)s;
}

long __semaphore_destroy(long handle) {
    if (!handle) return -1;
    vais_semaphore_t *s = (vais_semaphore_t *)handle;
    pthread_mutex_destroy(&s->mutex);
    pthread_cond_destroy(&s->cond);
    free(s);
    return 0;
}

long __semaphore_wait(long handle) {
    if (!handle) return -1;
    vais_semaphore_t *s = (vais_semaphore_t *)handle;

    pthread_mutex_lock(&s->mutex);
    while (s->permits <= 0) {
        pthread_cond_wait(&s->cond, &s->mutex);
    }
    s->permits--;
    pthread_mutex_unlock(&s->mutex);
    return 0;
}

long __semaphore_try_wait(long handle) {
    if (!handle) return 0;
    vais_semaphore_t *s = (vais_semaphore_t *)handle;

    pthread_mutex_lock(&s->mutex);
    if (s->permits > 0) {
        s->permits--;
        pthread_mutex_unlock(&s->mutex);
        return 1; // Acquired
    }
    pthread_mutex_unlock(&s->mutex);
    return 0; // Not acquired
}

long __semaphore_post(long handle) {
    if (!handle) return -1;
    vais_semaphore_t *s = (vais_semaphore_t *)handle;

    pthread_mutex_lock(&s->mutex);
    s->permits++;
    pthread_cond_signal(&s->cond);
    pthread_mutex_unlock(&s->mutex);
    return 0;
}

// ============================================
// Atomics (using GCC/Clang builtins)
// ============================================

long __atomic_load_i64(long ptr) {
    if (!ptr) return 0;
    return __atomic_load_n((long *)ptr, __ATOMIC_SEQ_CST);
}

long __atomic_store_i64(long ptr, long value) {
    if (!ptr) return -1;
    __atomic_store_n((long *)ptr, value, __ATOMIC_SEQ_CST);
    return 0;
}

long __atomic_exchange_i64(long ptr, long value) {
    if (!ptr) return 0;
    return __atomic_exchange_n((long *)ptr, value, __ATOMIC_SEQ_CST);
}

long __atomic_compare_exchange_i64(long ptr, long expected, long desired) {
    if (!ptr) return 0;
    long exp = expected;
    int ok = __atomic_compare_exchange_n(
        (long *)ptr, &exp, desired, 0,
        __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST
    );
    return ok ? 0 : 1; // 0 = success, 1 = failure
}

long __atomic_fetch_add_i64(long ptr, long value) {
    if (!ptr) return 0;
    return __atomic_fetch_add((long *)ptr, value, __ATOMIC_SEQ_CST);
}

long __atomic_fetch_sub_i64(long ptr, long value) {
    if (!ptr) return 0;
    return __atomic_fetch_sub((long *)ptr, value, __ATOMIC_SEQ_CST);
}

long __atomic_fetch_and_i64(long ptr, long value) {
    if (!ptr) return 0;
    return __atomic_fetch_and((long *)ptr, value, __ATOMIC_SEQ_CST);
}

long __atomic_fetch_or_i64(long ptr, long value) {
    if (!ptr) return 0;
    return __atomic_fetch_or((long *)ptr, value, __ATOMIC_SEQ_CST);
}

long __atomic_fetch_xor_i64(long ptr, long value) {
    if (!ptr) return 0;
    return __atomic_fetch_xor((long *)ptr, value, __ATOMIC_SEQ_CST);
}

// ============================================
// CPU hints
// ============================================

long __cpu_pause(void) {
#if defined(__x86_64__) || defined(__i386__)
    __asm__ __volatile__("pause");
#elif defined(__aarch64__)
    __asm__ __volatile__("yield");
#endif
    return 0;
}

// Memory helpers (__malloc, __free, __store_i64, __load_i64)
// are provided by http_runtime.c and codegen builtins respectively.
// Do not duplicate them here.
