// TLS runtime support for Vais
// Provides OpenSSL/LibreSSL FFI bindings for std/tls.vais
// Link with: -lssl -lcrypto
//
// Implements SSL_CTX and SSL lifecycle management,
// certificate loading, TLS handshake, and encrypted I/O.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/ssl.h>
#include <openssl/err.h>
#include <openssl/x509.h>
#include <openssl/x509v3.h>

// ============================================
// Global initialization (once)
// ============================================

static int tls_initialized = 0;

static void tls_global_init(void) {
    if (tls_initialized) return;
    SSL_library_init();
    SSL_load_error_strings();
    OpenSSL_add_all_algorithms();
    tls_initialized = 1;
}

// ============================================
// TLS Context (SSL_CTX)
// ============================================

// Create a new SSL_CTX for client (mode=0) or server (mode=1).
// Returns opaque pointer to SSL_CTX, or 0 on failure.
long __tls_ctx_new(long mode) {
    tls_global_init();

    const SSL_METHOD* method;
    if (mode == 1) {
        // Server
        method = TLS_server_method();
    } else {
        // Client
        method = TLS_client_method();
    }

    SSL_CTX* ctx = SSL_CTX_new(method);
    if (ctx == NULL) {
        return 0;
    }

    // Set reasonable defaults
    SSL_CTX_set_min_proto_version(ctx, TLS1_2_VERSION);
    SSL_CTX_set_options(ctx, SSL_OP_NO_SSLv2 | SSL_OP_NO_SSLv3);

    return (long)ctx;
}

// Free an SSL_CTX.
long __tls_ctx_free(long handle) {
    if (handle == 0) return -1;
    SSL_CTX_free((SSL_CTX*)handle);
    return 0;
}

// Load certificate chain from PEM file.
long __tls_ctx_load_cert(long handle, const char* path) {
    if (handle == 0 || path == NULL) return -3;
    SSL_CTX* ctx = (SSL_CTX*)handle;
    if (SSL_CTX_use_certificate_chain_file(ctx, path) != 1) {
        return -3;  // TLS_ERR_CERT
    }
    return 0;
}

// Load private key from PEM file.
long __tls_ctx_load_key(long handle, const char* path) {
    if (handle == 0 || path == NULL) return -4;
    SSL_CTX* ctx = (SSL_CTX*)handle;
    if (SSL_CTX_use_PrivateKey_file(ctx, path, SSL_FILETYPE_PEM) != 1) {
        return -4;  // TLS_ERR_KEY
    }
    // Verify key matches certificate
    if (SSL_CTX_check_private_key(ctx) != 1) {
        return -4;
    }
    return 0;
}

// Load CA certificate file for verification.
long __tls_ctx_load_ca(long handle, const char* path) {
    if (handle == 0 || path == NULL) return -5;
    SSL_CTX* ctx = (SSL_CTX*)handle;
    if (SSL_CTX_load_verify_locations(ctx, path, NULL) != 1) {
        return -5;  // TLS_ERR_CA
    }
    return 0;
}

// Load CA certificates from a directory.
long __tls_ctx_load_ca_dir(long handle, const char* path) {
    if (handle == 0 || path == NULL) return -5;
    SSL_CTX* ctx = (SSL_CTX*)handle;
    if (SSL_CTX_load_verify_locations(ctx, NULL, path) != 1) {
        return -5;  // TLS_ERR_CA
    }
    return 0;
}

// Set verification mode.
// mode 0 = SSL_VERIFY_NONE, mode 1 = SSL_VERIFY_PEER
long __tls_ctx_set_verify(long handle, long mode) {
    if (handle == 0) return -2;
    SSL_CTX* ctx = (SSL_CTX*)handle;
    if (mode == 1) {
        SSL_CTX_set_verify(ctx, SSL_VERIFY_PEER | SSL_VERIFY_FAIL_IF_NO_PEER_CERT, NULL);
    } else {
        SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL);
    }
    return 0;
}

// Load default system CA certificates for verification.
long __tls_ctx_set_default_verify(long handle) {
    if (handle == 0) return -2;
    SSL_CTX* ctx = (SSL_CTX*)handle;
    if (SSL_CTX_set_default_verify_paths(ctx) != 1) {
        // Non-fatal: system may not have default CA paths configured
        return -5;
    }
    return 0;
}

// ============================================
// TLS Connection (SSL)
// ============================================

// Create a new SSL object from an SSL_CTX and bind it to a socket fd.
// Returns opaque pointer to SSL, or 0 on failure.
long __tls_new(long ctx_handle, long fd) {
    if (ctx_handle == 0 || fd < 0) return 0;

    SSL_CTX* ctx = (SSL_CTX*)ctx_handle;
    SSL* ssl = SSL_new(ctx);
    if (ssl == NULL) return 0;

    if (SSL_set_fd(ssl, (int)fd) != 1) {
        SSL_free(ssl);
        return 0;
    }

    return (long)ssl;
}

// Free an SSL object.
long __tls_free(long handle) {
    if (handle == 0) return -1;
    SSL_free((SSL*)handle);
    return 0;
}

// Set SNI hostname for client connections.
long __tls_set_hostname(long handle, const char* hostname) {
    if (handle == 0 || hostname == NULL) return -10;
    SSL* ssl = (SSL*)handle;

    // Set SNI extension
    if (SSL_set_tlsext_host_name(ssl, hostname) != 1) {
        return -10;  // TLS_ERR_SNI
    }

    // Set hostname for certificate verification
    SSL_set1_host(ssl, hostname);

    return 0;
}

// Perform TLS client-side handshake (SSL_connect).
// Returns 0 on success, negative error code on failure.
long __tls_connect(long handle) {
    if (handle == 0) return -6;
    SSL* ssl = (SSL*)handle;

    int ret = SSL_connect(ssl);
    if (ret == 1) {
        return 0;  // Success
    }

    int err = SSL_get_error(ssl, ret);
    if (err == SSL_ERROR_WANT_READ) return -12;
    if (err == SSL_ERROR_WANT_WRITE) return -13;

    return -6;  // TLS_ERR_HANDSHAKE
}

// Perform TLS server-side handshake (SSL_accept).
// Returns 0 on success, negative error code on failure.
long __tls_accept(long handle) {
    if (handle == 0) return -6;
    SSL* ssl = (SSL*)handle;

    int ret = SSL_accept(ssl);
    if (ret == 1) {
        return 0;  // Success
    }

    int err = SSL_get_error(ssl, ret);
    if (err == SSL_ERROR_WANT_READ) return -12;
    if (err == SSL_ERROR_WANT_WRITE) return -13;

    return -6;  // TLS_ERR_HANDSHAKE
}

// Read decrypted data from TLS connection.
// Returns number of bytes read, or negative error code.
long __tls_read(long handle, long buf, long len) {
    if (handle == 0 || buf == 0 || len <= 0) return -7;
    SSL* ssl = (SSL*)handle;

    int ret = SSL_read(ssl, (void*)buf, (int)len);
    if (ret > 0) {
        return (long)ret;
    }

    int err = SSL_get_error(ssl, ret);
    if (err == SSL_ERROR_ZERO_RETURN) return 0;  // Clean shutdown
    if (err == SSL_ERROR_WANT_READ) return -12;
    if (err == SSL_ERROR_WANT_WRITE) return -13;

    return -7;  // TLS_ERR_READ
}

// Write data through TLS encryption.
// Returns number of bytes written, or negative error code.
long __tls_write(long handle, long data, long len) {
    if (handle == 0 || data == 0 || len <= 0) return -8;
    SSL* ssl = (SSL*)handle;

    int ret = SSL_write(ssl, (const void*)data, (int)len);
    if (ret > 0) {
        return (long)ret;
    }

    int err = SSL_get_error(ssl, ret);
    if (err == SSL_ERROR_WANT_READ) return -12;
    if (err == SSL_ERROR_WANT_WRITE) return -13;

    return -8;  // TLS_ERR_WRITE
}

// Shutdown TLS session (sends close_notify alert).
long __tls_shutdown(long handle) {
    if (handle == 0) return -9;
    SSL* ssl = (SSL*)handle;

    // SSL_shutdown may need to be called twice:
    // First call sends close_notify, second waits for peer's close_notify
    int ret = SSL_shutdown(ssl);
    if (ret == 0) {
        // Call again to complete bidirectional shutdown
        SSL_shutdown(ssl);
    }

    return 0;
}

// ============================================
// TLS Info Functions
// ============================================

// Get peer certificate's Common Name (CN).
// Returns pointer to a newly allocated string, or empty string.
const char* __tls_peer_cn(long handle) {
    if (handle == 0) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }

    SSL* ssl = (SSL*)handle;
    X509* cert = SSL_get_peer_certificate(ssl);
    if (cert == NULL) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }

    // Get subject name
    X509_NAME* subject = X509_get_subject_name(cert);
    if (subject == NULL) {
        X509_free(cert);
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }

    // Extract CN
    char cn_buf[256];
    int len = X509_NAME_get_text_by_NID(subject, NID_commonName, cn_buf, sizeof(cn_buf));
    X509_free(cert);

    if (len <= 0) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }

    char* result = (char*)malloc((size_t)len + 1);
    if (result) {
        memcpy(result, cn_buf, (size_t)len);
        result[len] = '\0';
    }
    return result;
}

// Get TLS protocol version string (e.g., "TLSv1.3").
const char* __tls_version(long handle) {
    if (handle == 0) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }

    SSL* ssl = (SSL*)handle;
    const char* ver = SSL_get_version(ssl);
    if (ver == NULL) ver = "";

    size_t len = strlen(ver);
    char* result = (char*)malloc(len + 1);
    if (result) {
        memcpy(result, ver, len);
        result[len] = '\0';
    }
    return result;
}

// Get cipher suite name (e.g., "TLS_AES_256_GCM_SHA384").
const char* __tls_cipher(long handle) {
    if (handle == 0) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }

    SSL* ssl = (SSL*)handle;
    const char* cipher = SSL_get_cipher_name(ssl);
    if (cipher == NULL) cipher = "";

    size_t len = strlen(cipher);
    char* result = (char*)malloc(len + 1);
    if (result) {
        memcpy(result, cipher, len);
        result[len] = '\0';
    }
    return result;
}
