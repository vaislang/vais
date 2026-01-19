# IPv6 Implementation for Net Module

## Overview
Added comprehensive IPv6 support to the `std/net.vais` module, enabling network programming with both IPv4 and IPv6 protocols.

## Implementation Date
2026-01-20

## Components Added

### 1. Constants

```vais
C AF_INET6: i64 = 30          # IPv6 address family (macOS: 30, Linux: 10)
C IPPROTO_IPV6: i64 = 41      # IPv6 protocol
C IPV6_V6ONLY: i64 = 27       # IPv6-only socket option (macOS)
C SOCKADDR_IN6_SIZE: i64 = 28 # sockaddr_in6 structure size
```

### 2. sockaddr_in6 Structure Support

The IPv6 socket address structure (28 bytes):
- `sin6_family` (2 bytes): Address family (AF_INET6)
- `sin6_port` (2 bytes): Port number (network byte order)
- `sin6_flowinfo` (4 bytes): IPv6 flow information
- `sin6_addr` (16 bytes): 128-bit IPv6 address
- `sin6_scope_id` (4 bytes): Scope ID for link-local addresses

#### Helper Functions

**`make_sockaddr_in6(host: *i8, port: i64) -> *sockaddr_in6`**
- Creates an IPv6 socket address structure
- Converts string IPv6 address to binary format using `inet_pton(AF_INET6, ...)`
- Returns allocated pointer (caller must free)

**`make_sockaddr_any6(port: i64) -> *sockaddr_in6`**
- Creates IPv6 wildcard address (::) for listening on all interfaces
- Convenience wrapper around `make_sockaddr_in6(0, port)`

### 3. TcpListener IPv6 Support

**`TcpListener.bind6(port: i64) -> TcpListener`**
- Creates and binds an IPv6 TCP listener
- Sets SO_REUSEADDR option for address reuse
- Listens on all IPv6 interfaces (::)
- Supports dual-stack mode by default (accepts IPv4 connections as ::ffff:x.x.x.x)

**C-Style API:**
```vais
tcp_listen6(port: i64) -> i64  # Returns file descriptor
```

### 4. TcpStream IPv6 Support

**`TcpStream.connect6(host: *i8, port: i64) -> TcpStream`**
- Connects to a remote IPv6 host
- Supports standard IPv6 addresses:
  - Loopback: `::1`
  - Link-local: `fe80::1`
  - Global unicast: `2001:db8::1`
  - IPv4-mapped: `::ffff:192.0.2.1`

**C-Style API:**
```vais
tcp_connect6(host: *i8, port: i64) -> i64  # Returns file descriptor
```

### 5. UdpSocket IPv6 Support

**`UdpSocket.new6() -> UdpSocket`**
- Creates an unbound IPv6 UDP socket

**`UdpSocket.bind6(port: i64) -> UdpSocket`**
- Creates and binds an IPv6 UDP socket to a port

**`socket.send_to6(data: *i8, len: i64, host: *i8, port: i64) -> i64`**
- Sends data to an IPv6 address
- Returns bytes sent or -1 on error

**`socket.recv_from6(buffer: *i8, len: i64, src_addr_out: *i8, src_port_out: *i64) -> i64`**
- Receives data with source IPv6 address information
- `src_addr_out` buffer should be at least 46 bytes (max IPv6 string length)
- Extracts source address and port from received packet

**C-Style API:**
```vais
udp_bind6(port: i64) -> i64
udp_send_to6(socket_fd: i64, data: *i8, len: i64, host: *i8, port: i64) -> i64
```

### 6. Utility Functions

**`is_valid_ip6(host: *i8) -> i64`**
- Validates IPv6 address string format
- Returns 1 if valid, 0 otherwise
- Uses `inet_pton(AF_INET6, ...)` for validation

## Platform Considerations

### macOS vs Linux
- `AF_INET6`: macOS = 30, Linux = 10
- Current implementation uses macOS value (30)
- For cross-platform support, this should be detected at compile-time or runtime

### Dual-Stack Behavior
- By default, IPv6 sockets accept both IPv4 and IPv6 connections
- IPv4 clients appear as IPv4-mapped IPv6 addresses: `::ffff:x.x.x.x`
- To force IPv6-only mode, set `IPV6_V6ONLY` socket option to 1

## Examples Created

### 1. `/examples/ipv6_test.vais`
Comprehensive test suite demonstrating:
- IPv6 address validation
- sockaddr_in6 structure creation
- TCP listener (IPv6)
- TCP client connection attempts
- UDP socket send/receive operations
- C-style API usage

### 2. `/examples/ipv6_dual_stack.vais`
Demonstrates dual-stack functionality:
- Creating listeners that accept both IPv4 and IPv6
- Setting IPV6_V6ONLY for IPv6-only mode
- Understanding IPv4-mapped IPv6 addresses

## API Consistency

All IPv6 functions follow the naming pattern of existing IPv4 functions with a `6` suffix:
- `bind()` → `bind6()`
- `connect()` → `connect6()`
- `send_to()` → `send_to6()`
- `recv_from()` → `recv_from6()`
- `tcp_listen()` → `tcp_listen6()`
- `tcp_connect()` → `tcp_connect6()`
- `udp_bind()` → `udp_bind6()`

## Documentation

### Updated Files
1. **`std/net.vais`**: Added 200+ lines of IPv6 implementation
2. **`ROADMAP.md`**: Marked IPv6 support as completed, added recent changes section
3. **`docs/STDLIB.md`**: Added comprehensive networking section with IPv6 documentation
4. **Examples**: Created `ipv6_test.vais` and `ipv6_dual_stack.vais`

### Module Index
Added `std/net` to the standard library module index with key types: `TcpListener`, `TcpStream`, `UdpSocket`

## Testing Recommendations

1. **IPv6 Address Validation**: Test with various IPv6 formats
2. **Loopback Connections**: Test TCP/UDP with `::1`
3. **Dual-Stack Mode**: Test IPv4 clients connecting to IPv6 sockets
4. **IPv6-Only Mode**: Test with IPV6_V6ONLY set
5. **Link-Local Addresses**: Test with `fe80::1` (may require scope ID)
6. **Cross-Platform**: Test on both macOS and Linux for AF_INET6 compatibility

## Future Enhancements

1. **Auto-detect AF_INET6**: Use platform detection instead of hardcoded value
2. **Scope ID Support**: Better handling of link-local addresses with scope IDs
3. **IPv6 Flow Information**: Support for sin6_flowinfo field
4. **Address Resolution**: DNS resolution with getaddrinfo for both IPv4/IPv6
5. **Network Interface Enumeration**: List available IPv6 interfaces

## Summary

This implementation provides complete parity between IPv4 and IPv6 networking in the Vais standard library. All core socket operations (TCP listen/connect, UDP bind/send/receive) now support both protocols, enabling modern network applications that can work across the full Internet Protocol spectrum.
