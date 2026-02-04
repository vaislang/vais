// Async runtime - macOS kqueue backend supplement for Vais
// Provides platform detection and stub functions for cross-platform compatibility.
// On macOS, kqueue/kevent/pipe/close are provided by the system.
// This file only adds the Vais-specific helpers.
//
// Compile on macOS: cc -c async_kqueue.c -o async_kqueue.o

#if defined(__APPLE__) || (defined(__unix__) && !defined(__linux__))

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/event.h>
#include <sys/time.h>
#include <time.h>

// ============================================
// Platform detection
// ============================================

// Returns 1 for macOS (kqueue)
long __async_platform(void) {
    return 1;
}

// ============================================
// Stub functions for cross-platform API compatibility
// These are no-ops on macOS since kqueue handles timers natively.
// ============================================

// epoll_set_timer_ms: not needed on macOS (kqueue has native timer support)
long __epoll_set_timer_ms(long kq, long timer_id, long delay_ms) {
    (void)kq;
    (void)timer_id;
    (void)delay_ms;
    return 0;
}

// iocp_set_timer_ms: not needed on macOS
long __iocp_set_timer_ms(long kq, long timer_id, long delay_ms) {
    (void)kq;
    (void)timer_id;
    (void)delay_ms;
    return 0;
}

// ============================================
// Helper functions that match the epoll/IOCP interface
// On macOS these wrap the system calls directly.
// ============================================

// __kevent_register: wrapper around kevent() for registering events
long __kevent_register(long kq, long fd, long filter, long flags) {
    struct kevent ev;
    unsigned short kev_flags = 0;

    if (flags & 1)  kev_flags |= EV_ADD;
    if (flags & 2)  kev_flags |= EV_DELETE;
    if (flags & 16) kev_flags |= EV_ONESHOT;

    EV_SET(&ev, (uintptr_t)fd, (int16_t)filter, kev_flags, 0, 0, NULL);

    int result = kevent((int)kq, &ev, 1, NULL, 0, NULL);
    return (long)result;
}

// __kevent_wait: wrapper around kevent() for waiting on events
long __kevent_wait(long kq, long events_buf, long max_events, long timeout_ms) {
    int max_ev = (int)max_events;
    if (max_ev > 256) max_ev = 256;

    struct kevent events[256];
    struct timespec ts;
    struct timespec* ts_ptr = NULL;

    if (timeout_ms >= 0) {
        ts.tv_sec = timeout_ms / 1000;
        ts.tv_nsec = (timeout_ms % 1000) * 1000000L;
        ts_ptr = &ts;
    }

    int n = kevent((int)kq, NULL, 0, events, max_ev, ts_ptr);
    if (n < 0) return 0;

    // Convert to Vais event format: [fd, filter] pairs
    long* buf = (long*)events_buf;
    for (int i = 0; i < n; i++) {
        buf[i * 2] = (long)events[i].ident;
        buf[i * 2 + 1] = (long)events[i].filter;
    }

    return (long)n;
}

// __kevent_get_fd: get fd from event at index
long __kevent_get_fd(long events_buf, long index) {
    long* buf = (long*)events_buf;
    return buf[index * 2];
}

// __kevent_get_filter: get filter from event at index
long __kevent_get_filter(long events_buf, long index) {
    long* buf = (long*)events_buf;
    return buf[index * 2 + 1];
}

// __write_byte: write a single byte to fd
long __write_byte(long fd, long value) {
    unsigned char byte = (unsigned char)(value & 0xFF);
    return (long)write((int)fd, &byte, 1);
}

// __read_byte: read a single byte from fd
long __read_byte(long fd) {
    unsigned char byte = 0;
    ssize_t n = read((int)fd, &byte, 1);
    if (n <= 0) return -1;
    return (long)byte;
}

// __time_now_ms: current time in milliseconds (monotonic)
long __time_now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long)(ts.tv_sec * 1000 + ts.tv_nsec / 1000000);
}

#endif // __APPLE__
