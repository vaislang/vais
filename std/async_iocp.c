// Async runtime - Windows IOCP backend for Vais
// Provides I/O Completion Port based async I/O as an alternative to kqueue/epoll.
// This file implements the same extern interface used by runtime.vais.
//
// Compile on Windows: cl.exe /c async_iocp.c /Fo:async_iocp.obj

#ifdef _WIN32

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>

#pragma comment(lib, "ws2_32.lib")

// ============================================
// IOCP backend - implements kqueue-compatible interface
// ============================================

// Vais filter constants:
//   EVFILT_READ  = -1
//   EVFILT_WRITE = -2
//   EVFILT_TIMER = -7
//   EV_ADD       = 1
//   EV_DELETE    = 2
//   EV_ONESHOT   = 16

// Internal: IOCP completion key encodes fd + filter
typedef struct {
    long fd;
    long filter;
} CompletionKey;

// Timer management
#define MAX_TIMERS 256

typedef struct {
    long timer_id;
    HANDLE timer_handle;
    HANDLE iocp;
    int active;
} TimerEntry;

static TimerEntry timer_table[MAX_TIMERS];
static int timer_table_initialized = 0;
static CRITICAL_SECTION timer_lock;

static void init_timer_table(void) {
    if (!timer_table_initialized) {
        memset(timer_table, 0, sizeof(timer_table));
        InitializeCriticalSection(&timer_lock);
        timer_table_initialized = 1;
    }
}

// IOCP event buffer layout:
// Each event: [fd: i64, filter: i64] = 16 bytes (matching kqueue format)

// Registered fd tracking for filter lookup
#define MAX_FDS 1024
typedef struct {
    long fd;
    long filter;
    int active;
} FdEntry;

static FdEntry fd_table[MAX_FDS];
static int fd_table_initialized = 0;

static void init_fd_table(void) {
    if (!fd_table_initialized) {
        memset(fd_table, 0, sizeof(fd_table));
        fd_table_initialized = 1;
    }
}

static void register_fd(long fd, long filter) {
    init_fd_table();
    for (int i = 0; i < MAX_FDS; i++) {
        if (!fd_table[i].active) {
            fd_table[i].fd = fd;
            fd_table[i].filter = filter;
            fd_table[i].active = 1;
            return;
        }
    }
}

static long get_filter_for_fd(long fd) {
    init_fd_table();
    for (int i = 0; i < MAX_FDS; i++) {
        if (fd_table[i].active && fd_table[i].fd == fd) {
            return fd_table[i].filter;
        }
    }
    return -1; // Default READ
}

static void unregister_fd(long fd) {
    init_fd_table();
    for (int i = 0; i < MAX_FDS; i++) {
        if (fd_table[i].active && fd_table[i].fd == fd) {
            fd_table[i].active = 0;
            return;
        }
    }
}

// Timer callback - posts completion to IOCP
static VOID CALLBACK timer_callback(PVOID lpParam, BOOLEAN TimerOrWaitFired) {
    (void)TimerOrWaitFired;
    TimerEntry* entry = (TimerEntry*)lpParam;
    if (entry && entry->active && entry->iocp) {
        // Post a completion packet with the timer_id as the completion key
        PostQueuedCompletionStatus(entry->iocp, 0, (ULONG_PTR)entry->timer_id, NULL);
    }
}

// ============================================
// kqueue-compatible API implemented with IOCP
// ============================================

// kqueue() -> create IOCP (kqueue-compatible name)
long kqueue(void) {
    // Initialize Winsock
    static int wsa_initialized = 0;
    if (!wsa_initialized) {
        WSADATA wsaData;
        WSAStartup(MAKEWORD(2, 2), &wsaData);
        wsa_initialized = 1;
    }

    init_timer_table();

    HANDLE iocp = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 1);
    if (iocp == NULL) {
        fprintf(stderr, "CreateIoCompletionPort failed: %lu\n", GetLastError());
        return -1;
    }
    return (long)(intptr_t)iocp;
}

// __kevent_register(kq, fd, filter, flags)
long __kevent_register(long kq, long fd, long filter, long flags) {
    HANDLE iocp = (HANDLE)(intptr_t)kq;

    if (filter == -7) {
        // EVFILT_TIMER
        init_timer_table();

        if (flags == 1 || flags == 17) {
            // EV_ADD - create timer
            EnterCriticalSection(&timer_lock);
            for (int i = 0; i < MAX_TIMERS; i++) {
                if (!timer_table[i].active) {
                    timer_table[i].timer_id = fd;  // timer_id stored in fd param
                    timer_table[i].iocp = iocp;
                    timer_table[i].active = 1;

                    // Timer will be armed by __iocp_set_timer_ms
                    LeaveCriticalSection(&timer_lock);
                    return 0;
                }
            }
            LeaveCriticalSection(&timer_lock);
            return -1; // Table full
        } else if (flags == 2) {
            // EV_DELETE - cancel timer
            EnterCriticalSection(&timer_lock);
            for (int i = 0; i < MAX_TIMERS; i++) {
                if (timer_table[i].active && timer_table[i].timer_id == fd) {
                    if (timer_table[i].timer_handle) {
                        DeleteTimerQueueTimer(NULL, timer_table[i].timer_handle, NULL);
                    }
                    timer_table[i].active = 0;
                    break;
                }
            }
            LeaveCriticalSection(&timer_lock);
            return 0;
        }
    }

    // Regular fd (socket) - associate with IOCP
    if (flags == 1 || flags == 17) {
        // EV_ADD
        register_fd(fd, filter);

        // Associate socket with IOCP
        HANDLE h = CreateIoCompletionPort((HANDLE)(intptr_t)fd, iocp, (ULONG_PTR)fd, 0);
        if (h == NULL) {
            // May already be associated, which is fine
            DWORD err = GetLastError();
            if (err != ERROR_INVALID_PARAMETER) {
                // Not already associated, real error
                // For sockets, we use WSARecv/WSASend with overlapped I/O
                // For now, we'll use polling via PostQueuedCompletionStatus
                PostQueuedCompletionStatus(iocp, 0, (ULONG_PTR)fd, NULL);
            }
        }
    } else if (flags == 2) {
        // EV_DELETE
        unregister_fd(fd);
    }

    return 0;
}

// Set timer delay in milliseconds (IOCP-specific helper)
long __iocp_set_timer_ms(long kq, long timer_id, long delay_ms) {
    init_timer_table();

    EnterCriticalSection(&timer_lock);
    for (int i = 0; i < MAX_TIMERS; i++) {
        if (timer_table[i].active && timer_table[i].timer_id == timer_id) {
            // Cancel existing timer if any
            if (timer_table[i].timer_handle) {
                DeleteTimerQueueTimer(NULL, timer_table[i].timer_handle, NULL);
                timer_table[i].timer_handle = NULL;
            }

            // Create new timer
            BOOL ok = CreateTimerQueueTimer(
                &timer_table[i].timer_handle,
                NULL,
                timer_callback,
                &timer_table[i],
                (DWORD)delay_ms,
                0,  // One-shot (period = 0)
                WT_EXECUTEONLYONCE
            );

            LeaveCriticalSection(&timer_lock);
            return ok ? 0 : -1;
        }
    }
    LeaveCriticalSection(&timer_lock);
    return -1;
}

// __kevent_wait(kq, events_buf, max_events, timeout_ms)
long __kevent_wait(long kq, long events_buf, long max_events, long timeout_ms) {
    HANDLE iocp = (HANDLE)(intptr_t)kq;
    long* buf = (long*)events_buf;
    int count = 0;
    int max_ev = (int)max_events;
    if (max_ev > 256) max_ev = 256;

    DWORD timeout = (timeout_ms < 0) ? INFINITE : (DWORD)timeout_ms;

    // Get at least one completion, then drain non-blocking
    for (int i = 0; i < max_ev; i++) {
        DWORD bytes;
        ULONG_PTR key;
        LPOVERLAPPED ov;
        DWORD wait_time = (i == 0) ? timeout : 0;

        BOOL ok = GetQueuedCompletionStatus(iocp, &bytes, &key, &ov, wait_time);

        if (!ok && ov == NULL) {
            // Timeout or error
            break;
        }

        long fd = (long)key;

        // Determine filter
        long filter;
        // Check if this is a timer completion
        int is_timer = 0;
        for (int j = 0; j < MAX_TIMERS; j++) {
            if (timer_table[j].active && timer_table[j].timer_id == fd) {
                is_timer = 1;
                break;
            }
        }

        if (is_timer) {
            filter = -7;  // EVFILT_TIMER
        } else {
            filter = get_filter_for_fd(fd);
        }

        buf[count * 2] = fd;
        buf[count * 2 + 1] = filter;
        count++;
    }

    return (long)count;
}

// __kevent_get_fd(events_buf, index)
long __kevent_get_fd(long events_buf, long index) {
    long* buf = (long*)events_buf;
    return buf[index * 2];
}

// __kevent_get_filter(events_buf, index)
long __kevent_get_filter(long events_buf, long index) {
    long* buf = (long*)events_buf;
    return buf[index * 2 + 1];
}

// ============================================
// Pipe emulation (Windows anonymous pipes)
// ============================================

// Windows doesn't have POSIX pipe(), provide compatible implementation
long pipe(long fds_buf) {
    HANDLE read_pipe, write_pipe;
    if (!CreatePipe(&read_pipe, &write_pipe, NULL, 0)) {
        fprintf(stderr, "CreatePipe failed: %lu\n", GetLastError());
        return -1;
    }
    long* buf = (long*)fds_buf;
    buf[0] = (long)(intptr_t)read_pipe;
    buf[1] = (long)(intptr_t)write_pipe;
    return 0;
}

// Windows doesn't have POSIX close(), provide compatible implementation
long close(long fd) {
    // Try both socket and handle close
    closesocket((SOCKET)(intptr_t)fd);
    CloseHandle((HANDLE)(intptr_t)fd);
    return 0;
}

long __write_byte(long fd, long value) {
    unsigned char byte = (unsigned char)(value & 0xFF);
    DWORD written;
    HANDLE h = (HANDLE)(intptr_t)fd;
    if (WriteFile(h, &byte, 1, &written, NULL)) {
        return (long)written;
    }
    // Try as socket
    return (long)send((SOCKET)(intptr_t)fd, (const char*)&byte, 1, 0);
}

long __read_byte(long fd) {
    unsigned char byte = 0;
    DWORD read_count;
    HANDLE h = (HANDLE)(intptr_t)fd;
    if (ReadFile(h, &byte, 1, &read_count, NULL) && read_count > 0) {
        return (long)byte;
    }
    // Try as socket
    int n = recv((SOCKET)(intptr_t)fd, (char*)&byte, 1, 0);
    if (n <= 0) return -1;
    return (long)byte;
}

// ============================================
// Time utilities
// ============================================

long __time_now_ms(void) {
    LARGE_INTEGER freq, counter;
    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&counter);
    return (long)((counter.QuadPart * 1000) / freq.QuadPart);
}

// ============================================
// Platform detection
// ============================================

// Returns 3 for Windows (IOCP)
long __async_platform(void) {
    return 3;
}

// ============================================
// Stub: epoll timer function (not used on Windows)
// ============================================

long __epoll_set_timer_ms(long kq, long timer_id, long delay_ms) {
    (void)kq;
    (void)timer_id;
    (void)delay_ms;
    return 0;
}

#endif // _WIN32
