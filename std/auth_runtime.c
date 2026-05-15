#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>
#include <time.h>

#if defined(__APPLE__)
#include <CommonCrypto/CommonHMAC.h>
#include <CommonCrypto/CommonKeyDerivation.h>
#elif defined(__linux__)
#include <openssl/hmac.h>
#include <openssl/evp.h>
#endif

#if defined(__linux__)
#include <errno.h>
#include <fcntl.h>
#include <sys/random.h>
#include <unistd.h>
#endif

static int64_t vais_auth_test_time_sec = -1;
static const char vais_auth_hex[] = "0123456789abcdef";

long __vais_auth_set_test_time_sec(long ts) {
    vais_auth_test_time_sec = (int64_t)ts;
    return 0;
}

long __vais_auth_clear_test_time_sec(void) {
    vais_auth_test_time_sec = -1;
    return 0;
}

long __vais_auth_current_time_sec(void) {
    if (vais_auth_test_time_sec >= 0) {
        return (long)vais_auth_test_time_sec;
    }
    return (long)time(NULL);
}

long __vais_auth_current_time_ms(void) {
    if (vais_auth_test_time_sec >= 0) {
        return (long)(vais_auth_test_time_sec * 1000);
    }

    struct timeval tv;
    if (gettimeofday(&tv, NULL) == 0) {
        return (long)((int64_t)tv.tv_sec * 1000 + (int64_t)tv.tv_usec / 1000);
    }
    return (long)((int64_t)time(NULL) * 1000);
}

static int vais_auth_fill_random(unsigned char *buf, size_t len) {
    if (len == 0) {
        return 0;
    }

#if defined(__APPLE__) || defined(__FreeBSD__) || defined(__OpenBSD__) || defined(__NetBSD__)
    arc4random_buf(buf, len);
    return 0;
#elif defined(__linux__)
    size_t offset = 0;
    while (offset < len) {
        ssize_t n = getrandom(buf + offset, len - offset, 0);
        if (n > 0) {
            offset += (size_t)n;
            continue;
        }
        if (n < 0 && errno == EINTR) {
            continue;
        }
        break;
    }
    if (offset == len) {
        return 0;
    }

    int fd = open("/dev/urandom", O_RDONLY);
    if (fd < 0) {
        return -1;
    }
    offset = 0;
    while (offset < len) {
        ssize_t n = read(fd, buf + offset, len - offset);
        if (n > 0) {
            offset += (size_t)n;
            continue;
        }
        if (n < 0 && errno == EINTR) {
            continue;
        }
        close(fd);
        return -1;
    }
    close(fd);
    return 0;
#else
    FILE *f = fopen("/dev/urandom", "rb");
    if (!f) {
        return -1;
    }
    size_t read_count = fread(buf, 1, len, f);
    fclose(f);
    return read_count == len ? 0 : -1;
#endif
}

char *__vais_auth_random_hex(long byte_len) {
    if (byte_len <= 0 || byte_len > 4096) {
        char *empty = (char *)malloc(1);
        if (empty) {
            empty[0] = '\0';
        }
        return empty;
    }

    size_t n = (size_t)byte_len;
    unsigned char *bytes = (unsigned char *)malloc(n);
    if (!bytes) {
        return NULL;
    }
    if (vais_auth_fill_random(bytes, n) != 0) {
        free(bytes);
        char *empty = (char *)malloc(1);
        if (empty) {
            empty[0] = '\0';
        }
        return empty;
    }

    char *out = (char *)malloc(n * 2 + 1);
    if (!out) {
        free(bytes);
        return NULL;
    }
    for (size_t i = 0; i < n; i++) {
        out[i * 2] = vais_auth_hex[(bytes[i] >> 4) & 0x0f];
        out[i * 2 + 1] = vais_auth_hex[bytes[i] & 0x0f];
    }
    out[n * 2] = '\0';
    free(bytes);
    return out;
}

char *__vais_auth_hmac_sha256_hex(const char *data, const char *secret) {
    if (!data || !secret) {
        return NULL;
    }

    unsigned char digest[32];
#if defined(__APPLE__)
    CCHmac(
        kCCHmacAlgSHA256,
        secret,
        strlen(secret),
        data,
        strlen(data),
        digest
    );
#elif defined(__linux__)
    unsigned int digest_len = 0;
    unsigned char *result = HMAC(
        EVP_sha256(),
        secret,
        (int)strlen(secret),
        (const unsigned char *)data,
        strlen(data),
        digest,
        &digest_len
    );
    if (!result || digest_len != 32) {
        return NULL;
    }
#else
    return NULL;
#endif

    char *out = (char *)malloc(65);
    if (!out) {
        return NULL;
    }
    for (int i = 0; i < 32; i++) {
        out[i * 2] = vais_auth_hex[(digest[i] >> 4) & 0x0f];
        out[i * 2 + 1] = vais_auth_hex[digest[i] & 0x0f];
    }
    out[64] = '\0';
    return out;
}

char *__vais_auth_pbkdf2_sha256_hex(const char *password, const char *salt, long iterations) {
    if (!password || !salt || iterations <= 0) {
        return NULL;
    }

    unsigned char digest[32];
#if defined(__APPLE__)
    int rc = CCKeyDerivationPBKDF(
        kCCPBKDF2,
        password,
        strlen(password),
        (const uint8_t *)salt,
        strlen(salt),
        kCCPRFHmacAlgSHA256,
        (uint)iterations,
        digest,
        sizeof(digest)
    );
    if (rc != 0) {
        return NULL;
    }
#elif defined(__linux__)
    int rc = PKCS5_PBKDF2_HMAC(
        password,
        (int)strlen(password),
        (const unsigned char *)salt,
        (int)strlen(salt),
        (int)iterations,
        EVP_sha256(),
        (int)sizeof(digest),
        digest
    );
    if (rc != 1) {
        return NULL;
    }
#else
    return NULL;
#endif

    char *out = (char *)malloc(65);
    if (!out) {
        return NULL;
    }
    for (int i = 0; i < 32; i++) {
        out[i * 2] = vais_auth_hex[(digest[i] >> 4) & 0x0f];
        out[i * 2 + 1] = vais_auth_hex[digest[i] & 0x0f];
    }
    out[64] = '\0';
    return out;
}
