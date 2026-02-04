# TLS API Reference

> TLS/SSL secure communication via OpenSSL/LibreSSL FFI

**Dependencies:** `-lssl -lcrypto`

## Import

```vais
U std/tls
```

## Features

- Client and server TLS contexts
- Certificate and private key loading (PEM)
- CA bundle for verification
- TLS handshake with SNI support
- Encrypted read/write

## Structs

### TlsContext

TLS configuration context.

| Method | Description |
|--------|-------------|
| `client()` | Create client context |
| `server()` | Create server context |
| `load_cert(path)` | Load certificate PEM |
| `load_key(path)` | Load private key PEM |
| `load_ca(path)` | Load CA bundle |
| `free()` | Free context |

### TlsConn

Active TLS connection.

| Method | Description |
|--------|-------------|
| `new(tcp_fd, ctx_handle)` | Create from TCP fd |
| `set_hostname(host)` | Set SNI hostname |
| `handshake()` | Perform TLS handshake |
| `read(buffer, len)` | Read decrypted data |
| `write(data, len)` | Write encrypted data |
| `write_str(s)` | Write string |
| `shutdown()` | Close TLS session |

## Usage

```vais
U std/tls
U std/net

F main() -> i64 {
    ctx := TlsContext::client()
    stream := TcpStream.connect("93.184.216.34", 443)
    conn := TlsConn::new(stream.fd, ctx.handle)
    conn.set_hostname("example.com")
    conn.handshake()
    conn.write_str("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
    buf := malloc(4096)
    n := conn.read(buf, 4096)
    conn.shutdown()
    ctx.free()
    0
}
```
