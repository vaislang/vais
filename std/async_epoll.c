// Async runtime - Linux epoll backend for Vais
// Provides epoll-based I/O multiplexing as an alternative to macOS kqueue.
// This file implements the same extern interface as kqueue but using epoll.
//
// Compile on Linux: gcc -c async_epoll.c -o async_epoll.o

#ifdef __linux__

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <sys/epoll.h>
#include <sys/timerfd.h>
#include <time.h>

// ============================================
// epoll backend - implements kqueue-compatible interface
// ============================================

// The Vais runtime.vais uses kqueue-style constants:
//   EVFILT_READ  = -1
//   EVFILT_WRITE = -2
//   EVFILT_TIMER = -7
//   EV_ADD       = 1
//   EV_DELETE    = 2
//   EV_ONESHOT   = 16
//
// This backend translates these to epoll equivalents.

// Internal mapping: timer_id -> timerfd
// Simple fixed-size table for timer management
#define MAX_TIMERS 256

static struct {
    long timer_id;
    int  timerfd;
    int  active;
} timer_table[MAX_TIMERS];

static int timer_table_initialized = 0;

static void init_timer_table(void) {
    if (!timer_table_initialized) {
        memset(timer_table, 0, sizeof(timer_table));
        timer_table_initialized = 1;
    }
}

static int find_or_add_timer(long timer_id, int timerfd) {
    init_timer_table();
    // Find existing
    for (int i = 0; i < MAX_TIMERS; i++) {
        if (timer_table[i].active && timer_table[i].timer_id == timer_id) {
            if (timerfd >= 0) timer_table[i].timerfd = timerfd;
            return i;
        }
    }
    // Add new
    if (timerfd >= 0) {
        for (int i = 0; i < MAX_TIMERS; i++) {
            if (!timer_table[i].active) {
                timer_table[i].timer_id = timer_id;
                timer_table[i].timerfd = timerfd;
                timer_table[i].active = 1;
                return i;
            }
        }
    }
    return -1;
}

static int get_timerfd_for_id(long timer_id) {
    init_timer_table();
    for (int i = 0; i < MAX_TIMERS; i++) {
        if (timer_table[i].active && timer_table[i].timer_id == timer_id) {
            return timer_table[i].timerfd;
        }
    }
    return -1;
}

static void remove_timer(long timer_id) {
    init_timer_table();
    for (int i = 0; i < MAX_TIMERS; i++) {
        if (timer_table[i].active && timer_table[i].timer_id == timer_id) {
            close(timer_table[i].timerfd);
            timer_table[i].active = 0;
            return;
        }
    }
}

// Reverse lookup: timerfd -> timer_id
static long get_timer_id_for_fd(int fd) {
    init_timer_table();
    for (int i = 0; i < MAX_TIMERS; i++) {
        if (timer_table[i].active && timer_table[i].timerfd == fd) {
            return timer_table[i].timer_id;
        }
    }
    return -1;
}

// ============================================
// kqueue-compatible API implemented with epoll
// ============================================

// kqueue() -> create epoll instance (kqueue-compatible name)
long kqueue(void) {
    int epfd = epoll_create1(0);
    if (epfd < 0) {
        perror("epoll_create1");
        return -1;
    }
    return (long)epfd;
}

// __kevent_register(kq, fd, filter, flags)
// filter: -1=READ, -2=WRITE, -7=TIMER
// flags: 1=ADD, 2=DELETE, 16=ONESHOT
long __kevent_register(long kq, long fd, long filter, long flags) {
    int epfd = (int)kq;

    if (filter == -7) {
        // EVFILT_TIMER: use timerfd
        if (flags == 1 || flags == 17) {
            // EV_ADD or EV_ADD|EV_ONESHOT
            // For timers, fd is the timer_id, and we need to create a timerfd
            // The delay is set via kevent_register with the timer_id as fd
            // We'll create the timerfd here and register it with epoll
            // The actual timeout will be set when we know the delay
            // For now, create a timerfd with a placeholder
            int tfd = timerfd_create(CLOCK_MONOTONIC, 0);
            if (tfd < 0) {
                perror("timerfd_create");
                return -1;
            }

            // Store mapping
            find_or_add_timer(fd, tfd);

            // Register timerfd with epoll
            struct epoll_event ev;
            ev.events = EPOLLIN;
            if (flags & 16) ev.events |= EPOLLONESHOT;
            ev.data.fd = tfd;

            if (epoll_ctl(epfd, EPOLL_CTL_ADD, tfd, &ev) < 0) {
                // Might already exist, try MOD
                if (epoll_ctl(epfd, EPOLL_CTL_MOD, tfd, &ev) < 0) {
                    perror("epoll_ctl timer");
                    close(tfd);
                    return -1;
                }
            }

            // Set the timer - use fd as milliseconds delay
            // Actually, in the Vais runtime, register_timer calls:
            //   kevent_register(kq, timer_id, EVFILT_TIMER, EV_ADD)
            // and the delay is passed separately. The kqueue backend uses
            // the NOTE_MSECONDS flag. For epoll, we need the delay value.
            // Since we don't have it here, we'll set a default and let
            // the reactor set it properly via a separate call.
            // For compatibility, use timer_id value as a hint.
            // The actual delay gets set by __epoll_set_timer_ms().

            return 0;
        } else if (flags == 2) {
            // EV_DELETE
            remove_timer(fd);
            return 0;
        }
    }

    // Regular fd (READ or WRITE)
    struct epoll_event ev;
    memset(&ev, 0, sizeof(ev));

    if (filter == -1) {
        ev.events = EPOLLIN;
    } else if (filter == -2) {
        ev.events = EPOLLOUT;
    } else {
        ev.events = EPOLLIN;
    }

    if (flags & 16) ev.events |= EPOLLONESHOT;

    ev.data.fd = (int)fd;

    if (flags == 1 || flags == 17) {
        // EV_ADD
        if (epoll_ctl(epfd, EPOLL_CTL_ADD, (int)fd, &ev) < 0) {
            if (errno == EEXIST) {
                epoll_ctl(epfd, EPOLL_CTL_MOD, (int)fd, &ev);
            } else {
                perror("epoll_ctl add");
                return -1;
            }
        }
    } else if (flags == 2) {
        // EV_DELETE
        epoll_ctl(epfd, EPOLL_CTL_DEL, (int)fd, NULL);
    }

    return 0;
}

// Set timer delay in milliseconds (epoll-specific helper)
// Called from the reactor to configure timerfd properly
long __epoll_set_timer_ms(long kq, long timer_id, long delay_ms) {
    int tfd = get_timerfd_for_id(timer_id);
    if (tfd < 0) return -1;

    struct itimerspec ts;
    memset(&ts, 0, sizeof(ts));
    ts.it_value.tv_sec = delay_ms / 1000;
    ts.it_value.tv_nsec = (delay_ms % 1000) * 1000000L;
    // One-shot: it_interval stays zero

    if (timerfd_settime(tfd, 0, &ts, NULL) < 0) {
        perror("timerfd_settime");
        return -1;
    }
    return 0;
}

// Internal buffer for event results
// Each event: [fd: i64, filter: i64] = 16 bytes
// The events_buf passed from Vais has this layout

// __kevent_wait(kq, events_buf, max_events, timeout_ms)
// Returns number of ready events
long __kevent_wait(long kq, long events_buf, long max_events, long timeout_ms) {
    int epfd = (int)kq;
    int max_ev = (int)max_events;
    if (max_ev > 256) max_ev = 256;

    struct epoll_event events[256];
    int timeout = (timeout_ms < 0) ? -1 : (int)timeout_ms;

    int n = epoll_wait(epfd, events, max_ev, timeout);
    if (n < 0) {
        if (errno == EINTR) return 0;
        perror("epoll_wait");
        return 0;
    }

    // Convert to Vais event format: [fd, filter] pairs
    long* buf = (long*)events_buf;
    for (int i = 0; i < n; i++) {
        int fd = events[i].data.fd;

        // Check if this fd is a timerfd -> translate back to timer_id
        long timer_id = get_timer_id_for_fd(fd);
        if (timer_id >= 0) {
            // Drain the timerfd
            uint64_t expirations;
            read(fd, &expirations, sizeof(expirations));

            buf[i * 2] = timer_id;       // Report timer_id, not timerfd
            buf[i * 2 + 1] = -7;         // EVFILT_TIMER
        } else {
            buf[i * 2] = (long)fd;

            // Determine filter from epoll events
            if (events[i].events & EPOLLIN) {
                buf[i * 2 + 1] = -1;     // EVFILT_READ
            } else if (events[i].events & EPOLLOUT) {
                buf[i * 2 + 1] = -2;     // EVFILT_WRITE
            } else {
                buf[i * 2 + 1] = -1;     // Default to READ
            }
        }
    }

    return (long)n;
}

// __kevent_get_fd(events_buf, index) -> fd at index
long __kevent_get_fd(long events_buf, long index) {
    long* buf = (long*)events_buf;
    return buf[index * 2];
}

// __kevent_get_filter(events_buf, index) -> filter at index
long __kevent_get_filter(long events_buf, long index) {
    long* buf = (long*)events_buf;
    return buf[index * 2 + 1];
}

// ============================================
// Pipe and I/O utilities
// ============================================

long __pipe(long fds_buf) {
    int pipefd[2];
    if (pipe(pipefd) < 0) {
        perror("pipe");
        return -1;
    }
    long* buf = (long*)fds_buf;
    buf[0] = (long)pipefd[0];  // read end
    buf[1] = (long)pipefd[1];  // write end
    return 0;
}

long __close(long fd) {
    return (long)close((int)fd);
}

long __write_byte(long fd, long value) {
    unsigned char byte = (unsigned char)(value & 0xFF);
    return (long)write((int)fd, &byte, 1);
}

long __read_byte(long fd) {
    unsigned char byte = 0;
    ssize_t n = read((int)fd, &byte, 1);
    if (n <= 0) return -1;
    return (long)byte;
}

// ============================================
// Time utilities
// ============================================

long __time_now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long)(ts.tv_sec * 1000 + ts.tv_nsec / 1000000);
}

// ============================================
// Platform detection
// ============================================

// Returns 2 for Linux (epoll)
long __async_platform(void) {
    return 2;
}

// ============================================
// Stub: IOCP timer function (not used on Linux)
// ============================================

long __iocp_set_timer_ms(long kq, long timer_id, long delay_ms) {
    (void)kq;
    (void)timer_id;
    (void)delay_ms;
    return 0;
}

#endif // __linux__
