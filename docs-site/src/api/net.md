# Net API Reference

> TCP/UDP socket networking with IPv4 and IPv6 support

## Import

```vais
U std/net
```

## Structs

### TcpListener

TCP server socket for accepting connections.

| Method | Signature | Description |
|--------|-----------|-------------|
| `bind` | `F bind(port: i64) -> TcpListener` | Bind IPv4 listener |
| `bind6` | `F bind6(port: i64) -> TcpListener` | Bind IPv6 listener |
| `accept` | `F accept(&self) -> TcpStream` | Accept connection |
| `close` | `F close(&self) -> i64` | Close listener |

### TcpStream

TCP connection for reading/writing data.

| Method | Signature | Description |
|--------|-----------|-------------|
| `connect` | `F connect(host: i64, port: i64) -> TcpStream` | Connect IPv4 |
| `connect6` | `F connect6(host: i64, port: i64) -> TcpStream` | Connect IPv6 |
| `read` | `F read(&self, buffer: i64, len: i64) -> i64` | Read data |
| `write` | `F write(&self, data: i64, len: i64) -> i64` | Write data |
| `write_all` | `F write_all(&self, data: i64, len: i64) -> i64` | Write all data |
| `close` | `F close(&self) -> i64` | Close connection |

### UdpSocket

UDP socket for connectionless datagram I/O.

| Method | Signature | Description |
|--------|-----------|-------------|
| `bind` | `F bind(port: i64) -> UdpSocket` | Bind IPv4 socket |
| `send_to` | `F send_to(&self, data: i64, len: i64, host: i64, port: i64) -> i64` | Send datagram |
| `recv` | `F recv(&self, buffer: i64, len: i64) -> i64` | Receive datagram |
| `close` | `F close(&self) -> i64` | Close socket |

## Convenience Functions

| Function | Description |
|----------|-------------|
| `tcp_listen(port)` | Create TCP listener, returns fd |
| `tcp_connect(host, port)` | Connect to TCP server, returns fd |
| `tcp_read(fd, buffer, len)` | Read from TCP socket |
| `tcp_write(fd, data, len)` | Write to TCP socket |
| `tcp_close(fd)` | Close TCP socket |
| `udp_bind(port)` | Bind UDP socket, returns fd |

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
