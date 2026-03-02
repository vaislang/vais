# Async Net API Reference

> Async TCP/UDP networking with non-blocking operations

## Import

```vais
U std/async_net
```

## Overview

The `async_net` module provides high-level async TCP and UDP networking APIs built on POSIX sockets with non-blocking I/O. It includes socket creation, address construction, and low-level socket operations.

## Constants

### Socket Types

| Constant | Value | Description |
|----------|-------|-------------|
| `AF_INET` | 2 | IPv4 address family |
| `SOCK_STREAM` | 1 | TCP socket |
| `SOCK_DGRAM` | 2 | UDP socket |
| `IPPROTO_TCP` | 6 | TCP protocol |
| `IPPROTO_UDP` | 17 | UDP protocol |

### Socket Options

| Constant | Value (macOS/Linux) | Description |
|----------|---------------------|-------------|
| `SOL_SOCKET` | 65535 / 1 | Socket level options |
| `SO_REUSEADDR` | 2 | Allow address reuse |
| `SO_BROADCAST` | 6 | Allow broadcast |
| `SO_KEEPALIVE` | 8 | Enable keepalive |

### Non-blocking I/O

| Constant | Value | Description |
|----------|-------|-------------|
| `F_GETFL` | 3 | Get file flags |
| `F_SETFL` | 4 | Set file flags |
| `O_NONBLOCK` | 4 | Non-blocking mode |
| `EAGAIN` | 35 | Would block |
| `EWOULDBLOCK` | 35 | Would block |
| `EINPROGRESS` | 36 | Connection in progress |

### Buffer Sizes

| Constant | Value | Description |
|----------|-------|-------------|
| `DEFAULT_READ_BUF_SIZE` | 8192 | Default read buffer |
| `MAX_UDP_PACKET` | 65536 | Maximum UDP packet size |

## Helper Functions

### make_sockaddr_in

```vais
F make_sockaddr_in(host: str, port: i64) -> i64
```

Create a `sockaddr_in` structure for the given host and port. Returns a pointer to the structure, or `0` on error.

## Extern Functions

The module exposes POSIX socket system calls: `socket`, `bind`, `listen`, `accept`, `connect`, `send`, `recv`, `sendto`, `recvfrom`, `close`, `setsockopt`, `getsockopt`, `inet_pton`, `htons`, `ntohs`, `fcntl`.

## Example

```vais
U std/async_net

F main() {
    # Create a TCP socket
    fd := socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)

    # Create address
    addr := make_sockaddr_in("127.0.0.1", 8080)

    # Bind and listen
    bind(fd, addr, SOCKADDR_IN_SIZE)
    listen(fd, 128)
}
```
