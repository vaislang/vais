// HTTP Client runtime support for Vais
// Provides TCP connect (client-side), DNS resolution, URL parsing,
// HTTP response parsing, timeout support, and SSL/TLS stubs
// for the std/http_client.vais standard library module.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/time.h>
#include <netinet/in.h>
#include <netdb.h>
#include <arpa/inet.h>
#include <ctype.h>
#include <limits.h>
#include <errno.h>

// ============================================
// Forward declarations for shared functions
// (defined in http_runtime.c)
// ============================================

// __find_header_end, __tcp_send, __tcp_recv, __tcp_close
// are provided by http_runtime.c and not duplicated here.

// ============================================
// Helper: duplicate a substring as a new C string
// ============================================

static char* hc_strndup(const char* s, size_t n) {
    char* result = (char*)malloc(n + 1);
    if (result) {
        memcpy(result, s, n);
        result[n] = '\0';
    }
    return result;
}

// ============================================
// DNS Resolution and TCP Connect
// ============================================

// Connect to a remote host with DNS resolution and optional timeout.
// host: pointer to hostname C string (e.g., "example.com" or "127.0.0.1")
// port: port number
// timeout_ms: connection timeout in milliseconds (0 = no timeout)
// Returns: socket file descriptor on success, -1 on failure.
long __hc_tcp_connect(long host, long port, long timeout_ms) {
    const char* hostname = (const char*)host;
    if (hostname == NULL || port <= 0 || port > 65535) return -1;

    // Resolve hostname via getaddrinfo
    struct addrinfo hints, *result, *rp;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;       // Allow IPv4 or IPv6
    hints.ai_socktype = SOCK_STREAM;   // TCP

    char port_str[16];
    snprintf(port_str, sizeof(port_str), "%ld", port);

    int ret = getaddrinfo(hostname, port_str, &hints, &result);
    if (ret != 0) {
        return -1;  // DNS resolution failed
    }

    // Try each resolved address
    int fd = -1;
    for (rp = result; rp != NULL; rp = rp->ai_next) {
        fd = socket(rp->ai_family, rp->ai_socktype, rp->ai_protocol);
        if (fd < 0) continue;

        // Set send/recv timeouts if specified
        if (timeout_ms > 0) {
            struct timeval tv;
            tv.tv_sec = timeout_ms / 1000;
            tv.tv_usec = (timeout_ms % 1000) * 1000;
            setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
            setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));
        }

        if (connect(fd, rp->ai_addr, rp->ai_addrlen) == 0) {
            break;  // Success
        }

        close(fd);
        fd = -1;
    }

    freeaddrinfo(result);
    return (long)fd;
}

// ============================================
// Resolve hostname to IP address string
// ============================================

// Resolve a hostname to an IPv4 address string.
// Returns pointer to a newly allocated string, or NULL on failure.
// Caller must free the result.
const char* __hc_resolve_host(const char* hostname) {
    if (hostname == NULL) return NULL;

    struct addrinfo hints, *result;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;

    int ret = getaddrinfo(hostname, NULL, &hints, &result);
    if (ret != 0) return NULL;

    char* ip_str = (char*)malloc(INET_ADDRSTRLEN);
    if (ip_str == NULL) {
        freeaddrinfo(result);
        return NULL;
    }

    struct sockaddr_in* addr = (struct sockaddr_in*)result->ai_addr;
    if (inet_ntop(AF_INET, &addr->sin_addr, ip_str, INET_ADDRSTRLEN) == NULL) {
        free(ip_str);
        freeaddrinfo(result);
        return NULL;
    }

    freeaddrinfo(result);
    return ip_str;
}

// ============================================
// Socket Timeout Support
// ============================================

// Set send and receive timeout on a socket.
// fd: socket file descriptor
// timeout_ms: timeout in milliseconds
// Returns: 0 on success, -1 on failure.
long __hc_set_timeout(long fd, long timeout_ms) {
    if (fd < 0 || timeout_ms <= 0) return -1;

    struct timeval tv;
    tv.tv_sec = timeout_ms / 1000;
    tv.tv_usec = (timeout_ms % 1000) * 1000;

    int ret1 = setsockopt((int)fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    int ret2 = setsockopt((int)fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));

    return (ret1 == 0 && ret2 == 0) ? 0 : -1;
}

// ============================================
// URL Parsing
// ============================================

// Helper: skip past "http://" or "https://" prefix
// Returns pointer to character after the scheme prefix
static const char* skip_scheme(const char* url) {
    if (url == NULL) return url;
    if (strncmp(url, "http://", 7) == 0) return url + 7;
    if (strncmp(url, "https://", 8) == 0) return url + 8;
    return url;  // No scheme prefix
}

// Parse scheme from URL ("http" or "https")
const char* __hc_parse_url_scheme(const char* url) {
    if (url == NULL) return hc_strndup("http", 4);
    if (strncmp(url, "https://", 8) == 0) return hc_strndup("https", 5);
    return hc_strndup("http", 4);
}

// Parse host from URL (e.g., "http://example.com:8080/path" -> "example.com")
const char* __hc_parse_url_host(const char* url) {
    if (url == NULL) {
        return hc_strndup("", 0);
    }

    const char* p = skip_scheme(url);
    const char* host_start = p;

    // Find end of host (: / ? or end of string)
    while (*p && *p != ':' && *p != '/' && *p != '?') p++;

    size_t len = (size_t)(p - host_start);
    if (len == 0) {
        return hc_strndup("", 0);
    }
    return hc_strndup(host_start, len);
}

// Parse port from URL. Returns 0 if no port specified.
long __hc_parse_url_port(const char* url) {
    if (url == NULL) return 0;

    const char* p = skip_scheme(url);

    // Skip host
    while (*p && *p != ':' && *p != '/' && *p != '?') p++;

    if (*p != ':') return 0;
    p++;  // skip ':'

    long port = 0;
    while (*p >= '0' && *p <= '9') {
        port = port * 10 + (*p - '0');
        p++;
    }
    return port;
}

// Parse path from URL (e.g., "http://example.com/api/v1?q=1" -> "/api/v1")
const char* __hc_parse_url_path(const char* url) {
    if (url == NULL) return hc_strndup("/", 1);

    const char* p = skip_scheme(url);

    // Skip host and optional port
    while (*p && *p != '/') p++;

    if (*p == '\0') return hc_strndup("/", 1);

    // Path goes from here to '?' or '#' or end
    const char* path_start = p;
    const char* path_end = p;
    while (*path_end && *path_end != '?' && *path_end != '#') path_end++;

    size_t len = (size_t)(path_end - path_start);
    if (len == 0) return hc_strndup("/", 1);
    return hc_strndup(path_start, len);
}

// Parse query string from URL (e.g., "http://example.com/path?key=val" -> "key=val")
// Returns empty string if no query string present.
const char* __hc_parse_url_query(const char* url) {
    if (url == NULL) return hc_strndup("", 0);

    const char* q = strchr(url, '?');
    if (q == NULL) return hc_strndup("", 0);

    q++;  // skip '?'

    // Query goes to '#' or end
    const char* end = q;
    while (*end && *end != '#') end++;

    size_t len = (size_t)(end - q);
    return hc_strndup(q, len);
}

// Full URL parsing into components:
// out_host: pointer to char* for host (caller must free)
// out_port: pointer to long for port
// out_path: pointer to char* for path (caller must free)
// Returns 0 on success, -1 on error.
long __hc_parse_url(const char* url, long out_host, long out_port, long out_path) {
    if (url == NULL) return -1;

    const char* host = __hc_parse_url_host(url);
    long port = __hc_parse_url_port(url);
    const char* path = __hc_parse_url_path(url);

    if (out_host != 0) *(const char**)out_host = host;
    if (out_port != 0) *(long*)out_port = port;
    if (out_path != 0) *(const char**)out_path = path;

    return 0;
}

// ============================================
// String to Integer Conversion
// ============================================

long __str_to_i64(const char* s) {
    if (s == NULL) return 0;
    long result = 0;
    int negative = 0;
    const char* p = s;

    // Skip whitespace
    while (*p == ' ' || *p == '\t') p++;

    // Handle sign
    if (*p == '-') { negative = 1; p++; }
    else if (*p == '+') { p++; }

    // Parse digits
    while (*p >= '0' && *p <= '9') {
        result = result * 10 + (*p - '0');
        p++;
    }

    return negative ? -result : result;
}

// ============================================
// Memset wrapper
// ============================================

long __memset(long dst, long value, long len) {
    if (dst != 0 && len > 0) {
        memset((void*)dst, (int)value, (size_t)len);
    }
    return dst;
}

// ============================================
// HTTP Response Parsing
// ============================================

// HttpResponse struct layout (must match std/http_client.vais and codegen):
// offset 0:  status (i64)
// offset 8:  status_text.ptr
// offset 16: status_text.len
// offset 24: version.ptr
// offset 32: version.len
// offset 40: headers (ptr - array of name/value pairs)
// offset 48: header_count (i64)
// offset 56: header_capacity (i64)
// offset 64: body (ptr)
// offset 72: body_len (i64)
// offset 80: error_code (i64)
// offset 88: ownership_mask (i64, emitted by codegen for string fields)

typedef struct {
    const char* ptr;
    long len;
} HcStr;

HcStr __hc_str_from_buffer(long ptr, long len) {
    HcStr out;
    out.ptr = (const char*)ptr;
    out.len = len > 0 ? len : 0;
    return out;
}

typedef struct {
    long status;
    HcStr status_text;
    HcStr version;
    long header_items;
    long header_count;
    long header_capacity;
    long body;
    long body_len;
    long error_code;
    long ownership_mask;
} HcResponse;

static HcStr hc_make_str(const char* start, size_t len) {
    HcStr out;
    out.ptr = hc_strndup(start, len);
    out.len = (long)len;
    return out;
}

static int hc_is_header_name_char(unsigned char ch) {
    return ch > 32 && ch < 127 && ch != ':';
}

static int hc_hex_digit_value(unsigned char ch) {
    if (ch >= '0' && ch <= '9') return ch - '0';
    if (ch >= 'A' && ch <= 'F') return ch - 'A' + 10;
    if (ch >= 'a' && ch <= 'f') return ch - 'a' + 10;
    return -1;
}

static const char* hc_body_start(const char* buf, const char* end) {
    const char* p = buf;
    while (p < end) {
        if (p + 3 < end && p[0] == '\r' && p[1] == '\n' && p[2] == '\r' && p[3] == '\n') {
            return p + 4;
        }
        if (p + 1 < end && p[0] == '\n' && p[1] == '\n') {
            return p + 2;
        }
        p++;
    }
    return NULL;
}

static int hc_contains_chunked_token(const char* value, size_t len) {
    const char* end = value + len;
    for (const char* p = value; p + 7 <= end; p++) {
        if (strncasecmp(p, "chunked", 7) == 0) {
            return 1;
        }
    }
    return 0;
}

static int hc_response_uses_chunked(const char* buf, const char* end) {
    const char* body = hc_body_start(buf, end);
    if (body == NULL) return 0;

    const char* p = buf;
    while (p < body && *p != '\r' && *p != '\n') p++;
    if (p < body && *p == '\r') p++;
    if (p < body && *p == '\n') p++;

    while (p < body) {
        if (*p == '\r' || *p == '\n') break;
        const char* line_start = p;
        const char* line_end = p;
        while (line_end < body && *line_end != '\r' && *line_end != '\n') line_end++;
        const char* colon = line_start;
        while (colon < line_end && *colon != ':') colon++;
        if (colon < line_end && (size_t)(colon - line_start) == 17
            && strncasecmp(line_start, "Transfer-Encoding", 17) == 0
            && hc_contains_chunked_token(colon + 1, (size_t)(line_end - colon - 1))) {
            return 1;
        }
        p = line_end;
        if (p < body && *p == '\r') p++;
        if (p < body && *p == '\n') p++;
    }
    return 0;
}

static int hc_consume_line_end(const char** cursor, const char* end) {
    const char* p = *cursor;
    if (p >= end) return 0;
    if (*p == '\r') {
        if (p + 1 >= end) return 0;
        if (p[1] != '\n') return -2;
        *cursor = p + 2;
        return 1;
    }
    if (*p == '\n') {
        *cursor = p + 1;
        return 1;
    }
    return -2;
}

static int hc_parse_chunked_body_bytes(
    const char* body,
    size_t body_len,
    char** decoded_out,
    size_t* decoded_len_out
) {
    char* decoded = NULL;
    size_t decoded_len = 0;
    if (decoded_out != NULL) {
        decoded = (char*)malloc(body_len + 1);
        if (decoded == NULL) return -2;
    }

    const char* p = body;
    const char* end = body + body_len;
    while (1) {
        size_t chunk_size = 0;
        int saw_digit = 0;

        while (p < end) {
            int digit = hc_hex_digit_value((unsigned char)*p);
            if (digit >= 0) {
                if (chunk_size > (((size_t)-1) - (size_t)digit) / 16) {
                    free(decoded);
                    return -2;
                }
                chunk_size = chunk_size * 16 + (size_t)digit;
                saw_digit = 1;
                p++;
                continue;
            }
            if (*p == ';') {
                while (p < end && *p != '\r' && *p != '\n') p++;
                break;
            }
            if (*p == '\r' || *p == '\n') break;
            free(decoded);
            return -2;
        }

        if (p >= end) {
            free(decoded);
            return 0;
        }
        if (!saw_digit) {
            free(decoded);
            return -2;
        }

        int line_end = hc_consume_line_end(&p, end);
        if (line_end != 1) {
            free(decoded);
            return line_end;
        }

        if (chunk_size == 0) {
            while (1) {
                if (p >= end) {
                    free(decoded);
                    return 0;
                }
                if (*p == '\r' || *p == '\n') {
                    line_end = hc_consume_line_end(&p, end);
                    if (line_end == 1) {
                        if (decoded != NULL) decoded[decoded_len] = '\0';
                        if (decoded_out != NULL) *decoded_out = decoded;
                        if (decoded_len_out != NULL) *decoded_len_out = decoded_len;
                        return 1;
                    }
                    free(decoded);
                    return line_end;
                }
                while (p < end && *p != '\r' && *p != '\n') p++;
                line_end = hc_consume_line_end(&p, end);
                if (line_end != 1) {
                    free(decoded);
                    return line_end;
                }
            }
        }

        if ((size_t)(end - p) < chunk_size) {
            free(decoded);
            return 0;
        }
        if (decoded != NULL) {
            memcpy(decoded + decoded_len, p, chunk_size);
        }
        decoded_len += chunk_size;
        p += chunk_size;

        line_end = hc_consume_line_end(&p, end);
        if (line_end != 1) {
            free(decoded);
            return line_end;
        }
    }
}

static HcResponse hc_parse_error_with_cleanup(HcResponse* out, long* items, long count) {
    if (items != NULL) {
        for (long i = 0; i < count; i++) {
            free((void*)items[i * 2]);
            free((void*)items[i * 2 + 1]);
        }
        free(items);
    }
    free((void*)out->status_text.ptr);
    free((void*)out->version.ptr);
    memset(out, 0, sizeof(HcResponse));
    out->error_code = -6;  // CLIENT_ERR_PARSE
    return *out;
}

// Parse an HTTP response from raw bytes.
// Returns HttpResponse struct by value to match the Vais external declaration.
HcResponse __hc_parse_response(long buffer, long len) {
    HcResponse out;
    memset(&out, 0, sizeof(HcResponse));

    const char* buf = (const char*)buffer;
    if (buf == NULL || len <= 0) {
        out.error_code = -6;  // CLIENT_ERR_PARSE
        return out;
    }

    const char* p = buf;
    const char* end = buf + len;

    // Validate and parse the status line before allocating response strings.
    if ((end - p) < 5 || strncmp(p, "HTTP/", 5) != 0) {
        out.error_code = -6;  // CLIENT_ERR_PARSE
        return out;
    }

    const char* ver_start = p;
    while (p < end && *p != ' ' && *p != '\r' && *p != '\n') p++;
    if (p >= end || *p != ' ' || p == ver_start) {
        out.error_code = -6;  // CLIENT_ERR_PARSE
        return out;
    }
    size_t ver_len = (size_t)(p - ver_start);

    p++;  // skip space before status code

    // HTTP status codes are exactly three digits.
    if ((end - p) < 3
        || p[0] < '0' || p[0] > '9'
        || p[1] < '0' || p[1] > '9'
        || p[2] < '0' || p[2] > '9') {
        out.error_code = -6;  // CLIENT_ERR_PARSE
        return out;
    }
    long status = 0;
    for (int i = 0; i < 3; i++) {
        status = status * 10 + (*p - '0');
        p++;
    }
    if (status < 100 || status > 999) {
        out.error_code = -6;  // CLIENT_ERR_PARSE
        return out;
    }

    if (p < end && *p != ' ' && *p != '\r' && *p != '\n') {
        out.error_code = -6;  // CLIENT_ERR_PARSE
        return out;
    }
    // Skip optional space before reason phrase.
    if (p < end && *p == ' ') p++;

    // Parse status text
    const char* text_start = p;
    while (p < end && *p != '\r' && *p != '\n') p++;
    if (p >= end) {
        out.error_code = -6;  // CLIENT_ERR_PARSE
        return out;
    }
    out.status = status;
    out.version = hc_make_str(ver_start, ver_len);
    out.status_text = hc_make_str(text_start, (size_t)(p - text_start));

    // Skip \r\n
    if (p < end && *p == '\r') p++;
    if (p < end && *p == '\n') p++;

    // Parse headers into allocated array
    long capacity = 16;
    long* items = (long*)malloc((size_t)(capacity * 16));
    long count = 0;
    int transfer_chunked = 0;

    while (p < end) {
        // Check for end of headers (\r\n or \n)
        if (*p == '\r' || *p == '\n') {
            if (p < end && *p == '\r') p++;
            if (p < end && *p == '\n') p++;
            break;
        }

        // Parse header name
        const char* name_start = p;
        const char* line_end = p;
        while (line_end < end && *line_end != '\r' && *line_end != '\n') line_end++;
        const char* colon = p;
        while (colon < line_end && *colon != ':') colon++;
        if (colon == line_end || colon == name_start) {
            return hc_parse_error_with_cleanup(&out, items, count);
        }
        for (const char* q = name_start; q < colon; q++) {
            if (!hc_is_header_name_char((unsigned char)*q)) {
                return hc_parse_error_with_cleanup(&out, items, count);
            }
        }
        p = colon;
        char* name = hc_strndup(name_start, (size_t)(p - name_start));

        // Skip ": "
        if (p < end) p++;  // skip ':'
        while (p < end && *p == ' ') p++;

        // Parse header value
        const char* val_start = p;
        while (p < end && *p != '\r' && *p != '\n') p++;
        char* value = hc_strndup(val_start, (size_t)(p - val_start));
        if (name != NULL && value != NULL
            && strcasecmp(name, "Transfer-Encoding") == 0
            && hc_contains_chunked_token(value, (size_t)(p - val_start))) {
            transfer_chunked = 1;
        }

        // Skip \r\n
        if (p < end && *p == '\r') p++;
        if (p < end && *p == '\n') p++;

        // Grow if needed
        if (count >= capacity) {
            capacity *= 2;
            items = (long*)realloc(items, (size_t)(capacity * 16));
        }

        // Store header (16 bytes: name ptr + value ptr)
        long offset = count * 2;  // 2 longs per header
        items[offset] = (long)name;
        items[offset + 1] = (long)value;
        count++;
    }

    out.header_items = (long)items;
    out.header_count = count;
    out.header_capacity = capacity;

    // Body is everything after headers
    // Make a copy so the caller can free the receive buffer independently
    if (p < end) {
        size_t body_len = (size_t)(end - p);
        if (transfer_chunked) {
            char* body_copy = NULL;
            size_t decoded_len = 0;
            int chunk_state = hc_parse_chunked_body_bytes(p, body_len, &body_copy, &decoded_len);
            if (chunk_state != 1) {
                return hc_parse_error_with_cleanup(&out, items, count);
            }
            out.body = (long)body_copy;
            out.body_len = (long)decoded_len;
        } else {
            char* body_copy = (char*)malloc(body_len + 1);
            if (body_copy) {
                memcpy(body_copy, p, body_len);
                body_copy[body_len] = '\0';
            }
            out.body = (long)body_copy;
            out.body_len = (long)body_len;
        }
    }

    out.error_code = 0;  // Success
    return out;
}

// ============================================
// Content-Length Extraction
// ============================================

// Extract Content-Length value from raw HTTP response headers.
// Returns the content length, -1 if not found, or -2 if malformed.
long __hc_get_content_length(long buffer, long len) {
    const char* buf = (const char*)buffer;
    if (buf == NULL || len <= 0) return -1;

    // Search for "Content-Length:" header (case-insensitive)
    const char* p = buf;
    const char* end = buf + len;

    while (p < end - 15) {
        // Check for \r\n or start of buffer preceding header name
        if (p == buf || (*(p - 1) == '\n')) {
            if (strncasecmp(p, "Content-Length:", 15) == 0) {
                p += 15;
                // Skip whitespace
                while (p < end && (*p == ' ' || *p == '\t')) p++;

                if (p >= end || *p < '0' || *p > '9') {
                    return -2;  // malformed Content-Length
                }

                // Parse number
                long cl = 0;
                while (p < end && *p >= '0' && *p <= '9') {
                    int digit = *p - '0';
                    if (cl > (LONG_MAX - digit) / 10) {
                        return -2;  // malformed Content-Length overflow
                    }
                    cl = cl * 10 + digit;
                    p++;
                }
                while (p < end && (*p == ' ' || *p == '\t')) p++;
                if (p < end && *p != '\r' && *p != '\n') {
                    return -2;  // malformed Content-Length suffix
                }
                return cl;
            }
        }
        p++;
    }

    return -1;  // Not found
}

// Return chunked response body state: 1 complete, 0 incomplete, -1 not chunked,
// or -2 malformed.
long __hc_chunked_body_complete(long buffer, long len) {
    const char* buf = (const char*)buffer;
    if (buf == NULL || len <= 0) return -1;

    const char* end = buf + len;
    const char* body = hc_body_start(buf, end);
    if (body == NULL) return 0;
    if (!hc_response_uses_chunked(buf, end)) return -1;

    return (long)hc_parse_chunked_body_bytes(body, (size_t)(end - body), NULL, NULL);
}

// ============================================
// SSL/TLS Stubs (Placeholder for Future)
// ============================================

// These functions provide a placeholder interface for future TLS support.
// Currently, they return error codes indicating TLS is not available.

// Initialize TLS context. Returns context pointer, or 0 on failure.
long __hc_tls_init(void) {
    // TLS not yet implemented
    return 0;
}

// Perform TLS handshake on an existing socket.
// ctx: TLS context from __hc_tls_init
// fd: connected socket file descriptor
// hostname: server hostname for SNI
// Returns: 0 on success, -1 on failure.
long __hc_tls_handshake(long ctx, long fd, const char* hostname) {
    (void)ctx;
    (void)fd;
    (void)hostname;
    // TLS not yet implemented
    return -1;
}

// Send data over TLS connection.
// Returns bytes sent, or -1 on error.
long __hc_tls_send(long ctx, long data, long len) {
    (void)ctx;
    (void)data;
    (void)len;
    return -1;
}

// Receive data over TLS connection.
// Returns bytes received, or -1 on error.
long __hc_tls_recv(long ctx, long buffer, long len) {
    (void)ctx;
    (void)buffer;
    (void)len;
    return -1;
}

// Close and free TLS context.
long __hc_tls_close(long ctx) {
    (void)ctx;
    return 0;
}
