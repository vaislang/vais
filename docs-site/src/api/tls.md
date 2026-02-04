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

## Constants

### Error Codes

| Constant | Value | Description |
|----------|-------|-------------|
| `TLS_OK` | 0 | Success |
| `TLS_ERR_INIT` | -1 | TLS initialization failed |
| `TLS_ERR_CTX` | -2 | Context error |
| `TLS_ERR_CERT` | -3 | Certificate loading failed |
| `TLS_ERR_KEY` | -4 | Private key loading failed |
| `TLS_ERR_CA` | -5 | CA certificate loading failed |
| `TLS_ERR_HANDSHAKE` | -6 | TLS handshake failed |
| `TLS_ERR_READ` | -7 | TLS read error |
| `TLS_ERR_WRITE` | -8 | TLS write error |
| `TLS_ERR_SHUTDOWN` | -9 | TLS shutdown error |
| `TLS_ERR_SNI` | -10 | SNI hostname setting failed |
| `TLS_ERR_VERIFY` | -11 | Certificate verification failed |
| `TLS_ERR_WANT_READ` | -12 | TLS wants read (non-blocking) |
| `TLS_ERR_WANT_WRITE` | -13 | TLS wants write (non-blocking) |

### TLS Mode Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `TLS_MODE_CLIENT` | 0 | Client mode |
| `TLS_MODE_SERVER` | 1 | Server mode |

### Verify Mode Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `TLS_VERIFY_NONE` | 0 | No verification |
| `TLS_VERIFY_PEER` | 1 | Verify peer certificate |

### Buffer Size Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `TLS_READ_BUF_SIZE` | 16384 | Default read buffer size |
| `TLS_MAX_HOSTNAME` | 256 | Maximum hostname length |

## Structs

### TlsContext

TLS configuration context.

| Method | Description |
|--------|-------------|
| `client()` | Create client context |
| `server()` | Create server context |
| `is_valid(&self)` | Check if context is valid |
| `load_cert(path)` | Load certificate PEM |
| `load_key(path)` | Load private key PEM |
| `load_ca(path)` | Load CA bundle file |
| `load_ca_dir(path)` | Load CA certificates from directory |
| `set_verify(mode)` | Set verify mode (NONE or PEER) |
| `insecure()` | Disable certificate verification |
| `free()` | Free context |

### TlsConn

Active TLS connection.

| Method | Description |
|--------|-------------|
| `new(tcp_fd, ctx_handle)` | Create from TCP fd |
| `is_valid(&self)` | Check if connection is valid |
| `set_hostname(host)` | Set SNI hostname |
| `handshake()` | Perform TLS client handshake |
| `accept()` | Perform TLS server handshake |
| `read(buffer, len)` | Read decrypted data |
| `write(data, len)` | Write encrypted data |
| `write_str(s)` | Write string |
| `read_str(max_len)` | Read into string |
| `peer_cn()` | Get peer certificate CN |
| `protocol_version()` | Get TLS protocol version |
| `cipher_name()` | Get cipher suite name |
| `shutdown()` | Close TLS session |
| `close()` | Close TLS and TCP socket |

## Helper Functions

| Function | Description |
|----------|-------------|
| `tls_connect(host, port)` | Connect via TLS (one-shot helper) |
| `tls_connect_timeout(host, port, timeout_ms)` | Connect via TLS with timeout |
| `tls_error_text(code)` | Convert error code to string |

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
