// WebSocket runtime support for Vais
// Provides SHA-1 hashing, Base64 encoding, WebSocket frame encode/decode,
// and handshake helpers per RFC 6455.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

// ============================================
// SHA-1 Implementation (RFC 3174)
// ============================================

static uint32_t sha1_rotl(uint32_t x, int n) {
    return (x << n) | (x >> (32 - n));
}

void __sha1(long input, long input_len, long output) {
    const uint8_t *msg = (const uint8_t *)input;
    size_t len = (size_t)input_len;
    uint8_t *hash = (uint8_t *)output;

    if (msg == NULL || hash == NULL || len < 0) return;

    uint32_t h0 = 0x67452301;
    uint32_t h1 = 0xEFCDAB89;
    uint32_t h2 = 0x98BADCFE;
    uint32_t h3 = 0x10325476;
    uint32_t h4 = 0xC3D2E1F0;

    // Pre-processing: adding padding bits
    // Message length in bits
    uint64_t bit_len = (uint64_t)len * 8;

    // Calculate padded length: original + 1 (0x80) + padding + 8 (length)
    // Padded length must be multiple of 64 bytes (512 bits)
    size_t padded_len = len + 1;
    while (padded_len % 64 != 56) {
        padded_len++;
    }
    padded_len += 8;

    uint8_t *padded = (uint8_t *)calloc(padded_len, 1);
    if (!padded) return;

    memcpy(padded, msg, len);
    padded[len] = 0x80;

    // Append length in bits as big-endian 64-bit
    for (int i = 0; i < 8; i++) {
        padded[padded_len - 1 - i] = (uint8_t)(bit_len >> (i * 8));
    }

    // Process each 64-byte block
    for (size_t offset = 0; offset < padded_len; offset += 64) {
        uint32_t w[80];

        // Break block into sixteen 32-bit big-endian words
        for (int i = 0; i < 16; i++) {
            w[i] = ((uint32_t)padded[offset + i * 4] << 24) |
                    ((uint32_t)padded[offset + i * 4 + 1] << 16) |
                    ((uint32_t)padded[offset + i * 4 + 2] << 8) |
                    ((uint32_t)padded[offset + i * 4 + 3]);
        }

        // Extend the sixteen 32-bit words into eighty 32-bit words
        for (int i = 16; i < 80; i++) {
            w[i] = sha1_rotl(w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16], 1);
        }

        uint32_t a = h0, b = h1, c = h2, d = h3, e = h4;

        for (int i = 0; i < 80; i++) {
            uint32_t f, k;
            if (i < 20) {
                f = (b & c) | ((~b) & d);
                k = 0x5A827999;
            } else if (i < 40) {
                f = b ^ c ^ d;
                k = 0x6ED9EBA1;
            } else if (i < 60) {
                f = (b & c) | (b & d) | (c & d);
                k = 0x8F1BBCDC;
            } else {
                f = b ^ c ^ d;
                k = 0xCA62C1D6;
            }

            uint32_t temp = sha1_rotl(a, 5) + f + e + k + w[i];
            e = d;
            d = c;
            c = sha1_rotl(b, 30);
            b = a;
            a = temp;
        }

        h0 += a;
        h1 += b;
        h2 += c;
        h3 += d;
        h4 += e;
    }

    free(padded);

    // Produce the final hash value (big-endian)
    hash[0]  = (h0 >> 24) & 0xFF; hash[1]  = (h0 >> 16) & 0xFF;
    hash[2]  = (h0 >> 8) & 0xFF;  hash[3]  = h0 & 0xFF;
    hash[4]  = (h1 >> 24) & 0xFF; hash[5]  = (h1 >> 16) & 0xFF;
    hash[6]  = (h1 >> 8) & 0xFF;  hash[7]  = h1 & 0xFF;
    hash[8]  = (h2 >> 24) & 0xFF; hash[9]  = (h2 >> 16) & 0xFF;
    hash[10] = (h2 >> 8) & 0xFF;  hash[11] = h2 & 0xFF;
    hash[12] = (h3 >> 24) & 0xFF; hash[13] = (h3 >> 16) & 0xFF;
    hash[14] = (h3 >> 8) & 0xFF;  hash[15] = h3 & 0xFF;
    hash[16] = (h4 >> 24) & 0xFF; hash[17] = (h4 >> 16) & 0xFF;
    hash[18] = (h4 >> 8) & 0xFF;  hash[19] = h4 & 0xFF;
}

// ============================================
// Base64 Encoding
// ============================================

static const char base64_table[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// Base64 encode input bytes, returns pointer to null-terminated string (caller must free)
long __base64_encode(long input, long input_len) {
    const uint8_t *data = (const uint8_t *)input;
    size_t len = (size_t)input_len;

    if (data == NULL || len == 0) {
        char *empty = (char *)malloc(1);
        if (empty) empty[0] = '\0';
        return (long)empty;
    }

    size_t out_len = 4 * ((len + 2) / 3);
    char *out = (char *)malloc(out_len + 1);
    if (!out) return 0;

    size_t i, j;
    for (i = 0, j = 0; i < len; ) {
        uint32_t a = i < len ? data[i++] : 0;
        uint32_t b = i < len ? data[i++] : 0;
        uint32_t c = i < len ? data[i++] : 0;

        uint32_t triple = (a << 16) | (b << 8) | c;

        out[j++] = base64_table[(triple >> 18) & 0x3F];
        out[j++] = base64_table[(triple >> 12) & 0x3F];
        out[j++] = base64_table[(triple >> 6) & 0x3F];
        out[j++] = base64_table[triple & 0x3F];
    }

    // Add padding
    size_t mod = len % 3;
    if (mod == 1) {
        out[out_len - 1] = '=';
        out[out_len - 2] = '=';
    } else if (mod == 2) {
        out[out_len - 1] = '=';
    }

    out[out_len] = '\0';
    return (long)out;
}

// ============================================
// WebSocket Accept Key Generation (RFC 6455)
// ============================================

// The magic GUID defined in RFC 6455
static const char *WS_MAGIC_GUID = "258EAFA5-E914-47DA-95CA-5AB5DC11D455";

// Compute the Sec-WebSocket-Accept value from the client's Sec-WebSocket-Key.
// Returns pointer to null-terminated base64 string (caller must free).
long __ws_accept_key(long client_key) {
    const char *key = (const char *)client_key;
    if (key == NULL) return 0;

    size_t key_len = strlen(key);
    size_t guid_len = strlen(WS_MAGIC_GUID);
    size_t concat_len = key_len + guid_len;

    char *concat = (char *)malloc(concat_len + 1);
    if (!concat) return 0;

    memcpy(concat, key, key_len);
    memcpy(concat + key_len, WS_MAGIC_GUID, guid_len);
    concat[concat_len] = '\0';

    // SHA-1 hash
    uint8_t hash[20];
    __sha1((long)concat, (long)concat_len, (long)hash);
    free(concat);

    // Base64 encode
    return __base64_encode((long)hash, 20);
}

// ============================================
// WebSocket Frame Encoding (RFC 6455)
// ============================================

// Frame format:
//  0                   1                   2                   3
//  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
// +-+-+-+-+-------+-+-------------+-------------------------------+
// |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
// |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
// |N|V|V|V|       |S|             |   (if payload len==126/127)   |
// | |1|2|3|       |K|             |                               |
// +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
// |     Extended payload length continued, if payload len == 127  |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-------------------------------+
// |                     Masking-key, if MASK set to 1             |
// +-------------------------------+-------------------------------+
// |                          Payload Data                         |
// +---------------------------------------------------------------+

// Encode a WebSocket frame.
// opcode: frame opcode (1=text, 2=binary, 8=close, 9=ping, 10=pong)
// payload: pointer to payload data
// payload_len: length of payload
// masked: 1 if frame should be masked (client->server), 0 otherwise
// mask_key: 4-byte mask key (only used if masked=1)
// out_frame: pointer to output buffer (must be large enough: payload_len + 14)
// Returns: total frame length written
long __ws_encode_frame(long opcode, long payload, long payload_len,
                       long masked, long mask_key, long out_frame) {
    uint8_t *frame = (uint8_t *)out_frame;
    const uint8_t *data = (const uint8_t *)payload;
    size_t len = (size_t)payload_len;
    size_t pos = 0;

    if (frame == NULL) return 0;

    // First byte: FIN=1, RSV=000, opcode
    frame[pos++] = 0x80 | ((uint8_t)opcode & 0x0F);

    // Second byte: MASK bit + payload length
    uint8_t mask_bit = masked ? 0x80 : 0x00;

    if (len <= 125) {
        frame[pos++] = mask_bit | (uint8_t)len;
    } else if (len <= 65535) {
        frame[pos++] = mask_bit | 126;
        frame[pos++] = (uint8_t)(len >> 8);
        frame[pos++] = (uint8_t)(len & 0xFF);
    } else {
        frame[pos++] = mask_bit | 127;
        for (int i = 7; i >= 0; i--) {
            frame[pos++] = (uint8_t)((len >> (i * 8)) & 0xFF);
        }
    }

    // Masking key (4 bytes if masked)
    if (masked) {
        uint8_t *mk = (uint8_t *)&mask_key;
        frame[pos++] = mk[0];
        frame[pos++] = mk[1];
        frame[pos++] = mk[2];
        frame[pos++] = mk[3];

        // Copy and mask payload
        if (data != NULL && len > 0) {
            for (size_t i = 0; i < len; i++) {
                frame[pos + i] = data[i] ^ mk[i % 4];
            }
        }
    } else {
        // Copy payload unmasked
        if (data != NULL && len > 0) {
            memcpy(frame + pos, data, len);
        }
    }

    return (long)(pos + len);
}

// ============================================
// WebSocket Frame Decoding
// ============================================

// Decoded frame output structure (matches Vais WsFrame struct):
// offset 0:  opcode (i64)
// offset 8:  payload (i64 - pointer)
// offset 16: payload_len (i64)
// offset 24: is_final (i64)
// offset 32: is_masked (i64)

// Decode a WebSocket frame from raw data.
// data: pointer to raw frame bytes
// data_len: length of available data
// out_frame: pointer to WsFrame struct to fill
// Returns: number of bytes consumed (total frame size), or 0 if incomplete, -1 on error
long __ws_decode_frame(long data, long data_len, long out_frame) {
    const uint8_t *buf = (const uint8_t *)data;
    size_t len = (size_t)data_len;
    long *frame_out = (long *)out_frame;

    if (buf == NULL || len < 2 || frame_out == NULL) return -1;

    size_t pos = 0;

    // First byte: FIN + opcode
    uint8_t byte0 = buf[pos++];
    int is_final = (byte0 & 0x80) ? 1 : 0;
    int opcode = byte0 & 0x0F;

    // Second byte: MASK + payload length
    uint8_t byte1 = buf[pos++];
    int is_masked = (byte1 & 0x80) ? 1 : 0;
    uint64_t payload_len = byte1 & 0x7F;

    if (payload_len == 126) {
        if (len < pos + 2) return 0;  // Incomplete
        payload_len = ((uint64_t)buf[pos] << 8) | (uint64_t)buf[pos + 1];
        pos += 2;
    } else if (payload_len == 127) {
        if (len < pos + 8) return 0;  // Incomplete
        payload_len = 0;
        for (int i = 0; i < 8; i++) {
            payload_len = (payload_len << 8) | (uint64_t)buf[pos + i];
        }
        pos += 8;
    }

    // Masking key
    uint8_t mask_key[4] = {0};
    if (is_masked) {
        if (len < pos + 4) return 0;  // Incomplete
        mask_key[0] = buf[pos++];
        mask_key[1] = buf[pos++];
        mask_key[2] = buf[pos++];
        mask_key[3] = buf[pos++];
    }

    // Check if we have the full payload
    if (len < pos + payload_len) return 0;  // Incomplete

    // Allocate and copy/unmask payload
    uint8_t *payload = NULL;
    if (payload_len > 0) {
        payload = (uint8_t *)malloc((size_t)payload_len + 1);
        if (!payload) return -1;

        if (is_masked) {
            for (uint64_t i = 0; i < payload_len; i++) {
                payload[i] = buf[pos + i] ^ mask_key[i % 4];
            }
        } else {
            memcpy(payload, buf + pos, (size_t)payload_len);
        }
        payload[payload_len] = '\0';  // Null-terminate for text frames
    }

    // Fill output struct
    frame_out[0] = (long)opcode;        // opcode
    frame_out[1] = (long)payload;       // payload pointer
    frame_out[2] = (long)payload_len;   // payload_len
    frame_out[3] = (long)is_final;      // is_final
    frame_out[4] = (long)is_masked;     // is_masked

    return (long)(pos + (size_t)payload_len);
}

// ============================================
// WebSocket Masking/Unmasking
// ============================================

// Apply XOR mask to data (in-place). Works for both masking and unmasking.
// data: pointer to data buffer
// len: length of data
// mask_key: 4-byte mask key as i64 (lower 4 bytes)
long __ws_mask(long data, long len, long mask_key) {
    uint8_t *buf = (uint8_t *)data;
    size_t length = (size_t)len;
    uint8_t *mk = (uint8_t *)&mask_key;

    if (buf == NULL || length == 0) return 0;

    for (size_t i = 0; i < length; i++) {
        buf[i] ^= mk[i % 4];
    }

    return 0;
}

// Unmask is identical to mask (XOR is its own inverse)
long __ws_unmask(long data, long len, long mask_key) {
    return __ws_mask(data, len, mask_key);
}

// ============================================
// HTTP Upgrade Request Parsing
// ============================================

// Parse an HTTP upgrade request and extract the Sec-WebSocket-Key header value.
// buffer: pointer to raw HTTP request data
// len: length of data
// Returns: pointer to extracted key string (caller must free), or 0 if not found
long __ws_parse_upgrade_request(long buffer, long len) {
    const char *buf = (const char *)buffer;
    size_t length = (size_t)len;

    if (buf == NULL || length < 4) return 0;

    // Search for "Sec-WebSocket-Key:" header (case-insensitive)
    const char *target = "Sec-WebSocket-Key:";
    size_t target_len = 18;

    for (size_t i = 0; i <= length - target_len; i++) {
        int match = 1;
        for (size_t j = 0; j < target_len; j++) {
            char a = buf[i + j];
            char b = target[j];
            // Case-insensitive compare for letters
            if (a >= 'A' && a <= 'Z') a += 32;
            if (b >= 'A' && b <= 'Z') b += 32;
            if (a != b) {
                match = 0;
                break;
            }
        }

        if (match) {
            // Found the header, extract value
            size_t val_start = i + target_len;

            // Skip whitespace
            while (val_start < length && buf[val_start] == ' ') {
                val_start++;
            }

            // Find end of value (CR or LF)
            size_t val_end = val_start;
            while (val_end < length && buf[val_end] != '\r' && buf[val_end] != '\n') {
                val_end++;
            }

            // Trim trailing whitespace
            while (val_end > val_start && buf[val_end - 1] == ' ') {
                val_end--;
            }

            size_t val_len = val_end - val_start;
            if (val_len == 0) return 0;

            char *key = (char *)malloc(val_len + 1);
            if (!key) return 0;
            memcpy(key, buf + val_start, val_len);
            key[val_len] = '\0';

            return (long)key;
        }
    }

    return 0;
}

// ============================================
// WebSocket Upgrade Response Builder
// ============================================

// Build the HTTP 101 Switching Protocols response for WebSocket upgrade.
// accept_key: the computed Sec-WebSocket-Accept value
// out_buffer: output buffer for the response
// Returns: length of response written
long __ws_build_upgrade_response(long accept_key, long out_buffer) {
    const char *key = (const char *)accept_key;
    char *buf = (char *)out_buffer;

    if (key == NULL || buf == NULL) return 0;

    size_t pos = 0;

    // Status line
    const char *status = "HTTP/1.1 101 Switching Protocols\r\n";
    size_t slen = strlen(status);
    memcpy(buf + pos, status, slen);
    pos += slen;

    // Upgrade header
    const char *upgrade = "Upgrade: websocket\r\n";
    slen = strlen(upgrade);
    memcpy(buf + pos, upgrade, slen);
    pos += slen;

    // Connection header
    const char *conn = "Connection: Upgrade\r\n";
    slen = strlen(conn);
    memcpy(buf + pos, conn, slen);
    pos += slen;

    // Sec-WebSocket-Accept header
    const char *accept_hdr = "Sec-WebSocket-Accept: ";
    slen = strlen(accept_hdr);
    memcpy(buf + pos, accept_hdr, slen);
    pos += slen;

    size_t key_len = strlen(key);
    memcpy(buf + pos, key, key_len);
    pos += key_len;

    // CRLF after accept header
    buf[pos++] = '\r';
    buf[pos++] = '\n';

    // End of headers
    buf[pos++] = '\r';
    buf[pos++] = '\n';

    return (long)pos;
}

// ============================================
// Utility: Generate a simple pseudo-random mask key
// ============================================

static uint32_t ws_rand_state = 12345;

long __ws_random_mask_key(void) {
    // Simple LCG PRNG for mask key generation
    ws_rand_state = ws_rand_state * 1103515245 + 12345;
    uint32_t a = ws_rand_state;
    ws_rand_state = ws_rand_state * 1103515245 + 12345;
    uint32_t b = ws_rand_state;
    return (long)((a & 0xFFFF0000) | (b >> 16));
}

// ============================================
// Debug / Logging
// ============================================

long __ws_log(const char *msg) {
    if (msg != NULL) {
        fprintf(stderr, "[WebSocket] %s\n", msg);
    }
    return 0;
}

long __ws_log_int(const char *label, long value) {
    if (label != NULL) {
        fprintf(stderr, "[WebSocket] %s: %ld\n", label, value);
    }
    return 0;
}
