# Net API Reference

> TCP/UDP socket networking with IPv4 and IPv6 support

## Import

```vais
U std/net
```

## Constants

### Error Codes

| Constant | Value | Description |
|----------|-------|-------------|
| `NET_ERR_NONE` | 0 | Success |
| `NET_ERR_SOCKET` | -1 | Socket creation failed |
| `NET_ERR_BIND` | -2 | Bind failed |
| `NET_ERR_LISTEN` | -3 | Listen failed |
| `NET_ERR_ACCEPT` | -4 | Accept failed |
| `NET_ERR_CONNECT` | -5 | Connect failed |
| `NET_ERR_SEND` | -6 | Send failed |
| `NET_ERR_RECV` | -7 | Receive failed |
| `NET_ERR_CLOSE` | -8 | Close failed |
| `NET_ERR_INVALID` | -9 | Invalid argument |
| `NET_ERR_RESOLVE` | -10 | Address resolution failed |
| `NET_ERR_INVALID_PORT` | -11 | Invalid port number |
| `NET_ERR_INVALID_BUFFER` | -12 | Invalid buffer |

### Socket Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `AF_INET` | 2 | IPv4 address family |
| `AF_INET6` | 30 | IPv6 address family |
| `SOCK_STREAM` | 1 | TCP socket type |
| `SOCK_DGRAM` | 2 | UDP socket type |
| `MIN_PORT` | 0 | Minimum port number |
| `MAX_PORT` | 65535 | Maximum port number |

## Structs

### TcpListener

TCP server socket for accepting connections.

| Method | Signature | Description |
|--------|-----------|-------------|
| `bind` | `F bind(port: i64) -> TcpListener` | Bind IPv4 listener |
| `bind6` | `F bind6(port: i64) -> TcpListener` | Bind IPv6 listener |
| `is_valid` | `F is_valid(&self) -> i64` | Check if listener is valid |
| `accept` | `F accept(&self) -> TcpStream` | Accept connection |
| `get_port` | `F get_port(&self) -> i64` | Get listening port |
| `close` | `F close(&self) -> i64` | Close listener |
| `drop` | `F drop(&self) -> i64` | Close listener (RAII) |

### TcpStream

TCP connection for reading/writing data.

| Method | Signature | Description |
|--------|-----------|-------------|
| `connect` | `F connect(host: i64, port: i64) -> TcpStream` | Connect IPv4 |
| `connect6` | `F connect6(host: i64, port: i64) -> TcpStream` | Connect IPv6 |
| `is_valid` | `F is_valid(&self) -> i64` | Check if stream is valid |
| `read` | `F read(&self, buffer: i64, len: i64) -> i64` | Read data |
| `write` | `F write(&self, data: i64, len: i64) -> i64` | Write data |
| `write_all` | `F write_all(&self, data: i64, len: i64) -> i64` | Write all data |
| `get_fd` | `F get_fd(&self) -> i64` | Get file descriptor |
| `close` | `F close(&self) -> i64` | Close connection |
| `drop` | `F drop(&self) -> i64` | Close connection (RAII) |

### UdpSocket

UDP socket for connectionless datagram I/O.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> UdpSocket` | Create unbound IPv4 socket |
| `new6` | `F new6() -> UdpSocket` | Create unbound IPv6 socket |
| `bind` | `F bind(port: i64) -> UdpSocket` | Bind IPv4 socket |
| `bind6` | `F bind6(port: i64) -> UdpSocket` | Bind IPv6 socket |
| `is_valid` | `F is_valid(&self) -> i64` | Check if socket is valid |
| `send_to` | `F send_to(&self, data: i64, len: i64, host: i64, port: i64) -> i64` | Send datagram (IPv4) |
| `send_to6` | `F send_to6(&self, data: i64, len: i64, host: i64, port: i64) -> i64` | Send datagram (IPv6) |
| `recv` | `F recv(&self, buffer: i64, len: i64) -> i64` | Receive datagram |
| `recv_from` | `F recv_from(&self, buffer: i64, len: i64, src_addr_out: i64, src_port_out: i64) -> i64` | Receive with source (IPv4) |
| `recv_from6` | `F recv_from6(&self, buffer: i64, len: i64, src_addr_out: i64, src_port_out: i64) -> i64` | Receive with source (IPv6) |
| `get_port` | `F get_port(&self) -> i64` | Get bound port |
| `get_fd` | `F get_fd(&self) -> i64` | Get file descriptor |
| `close` | `F close(&self) -> i64` | Close socket |
| `drop` | `F drop(&self) -> i64` | Close socket (RAII) |

## Convenience Functions

### TCP Functions

| Function | Description |
|----------|-------------|
| `tcp_listen(port)` | Create TCP listener (IPv4), returns fd |
| `tcp_listen6(port)` | Create TCP listener (IPv6), returns fd |
| `tcp_listen_result(port)` | Create TCP listener with Result (IPv4) |
| `tcp_listen6_result(port)` | Create TCP listener with Result (IPv6) |
| `tcp_accept(listener_fd)` | Accept connection, returns client fd |
| `tcp_accept_result(listener_fd)` | Accept connection with Result |
| `tcp_close_listener(listener_fd)` | Close TCP listener |
| `tcp_connect(host, port)` | Connect to TCP server (IPv4), returns fd |
| `tcp_connect6(host, port)` | Connect to TCP server (IPv6), returns fd |
| `tcp_connect_result(host, port)` | Connect with Result (IPv4) |
| `tcp_connect6_result(host, port)` | Connect with Result (IPv6) |
| `tcp_read(fd, buffer, len)` | Read from TCP socket |
| `tcp_write(fd, data, len)` | Write to TCP socket |
| `tcp_close(fd)` | Close TCP socket |

### UDP Functions

| Function | Description |
|----------|-------------|
| `udp_bind(port)` | Bind UDP socket (IPv4), returns fd |
| `udp_bind6(port)` | Bind UDP socket (IPv6), returns fd |
| `udp_send_to(fd, data, len, host, port)` | Send UDP datagram (IPv4) |
| `udp_send_to6(fd, data, len, host, port)` | Send UDP datagram (IPv6) |
| `udp_recv_from(fd, buffer, len)` | Receive UDP datagram |
| `udp_close(fd)` | Close UDP socket |

### Utility Functions

| Function | Description |
|----------|-------------|
| `is_valid_ip(host)` | Check if IPv4 address is valid |
| `is_valid_ip6(host)` | Check if IPv6 address is valid |
| `net_error_string(err)` | Convert error code to string |

## Usage

```vais
U std/net

# TCP Server
listener := TcpListener.bind(8080)
client := listener.accept()
buf := malloc(1024)
n := client.read(buf, 1024)
client.close()
listener.close()
```
