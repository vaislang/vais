// Compression runtime support for Vais
// Provides zlib FFI bindings for std/compress.vais
// Link with: -lz
//
// Implements deflate (RFC 1951) and gzip (RFC 1952) compression
// and decompression with streaming support.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <zlib.h>

// ============================================
// Constants
// ============================================

#define COMPRESS_OK 0
#define COMPRESS_ERR_INIT -1
#define COMPRESS_ERR_PARAM -2
#define COMPRESS_ERR_MEMORY -3
#define COMPRESS_ERR_DATA -4
#define COMPRESS_ERR_STREAM -5
#define COMPRESS_ERR_VERSION -6
#define COMPRESS_ERR_BUFFER -7

#define COMPRESS_DEFLATE 0
#define COMPRESS_GZIP 1

#define COMPRESS_CHUNK_SIZE 16384
#define COMPRESS_INITIAL_BUFSIZE 32768

// ============================================
// Compressor State
// ============================================

typedef struct {
    z_stream stream;
    int mode;           // COMPRESS_DEFLATE or COMPRESS_GZIP
    int level;          // 1-9
    int initialized;    // 1 if stream is initialized
    int streaming;      // 1 if in streaming mode
} compressor_t;

// ============================================
// Helper Functions
// ============================================

// Convert zlib error code to our error code
static long zlib_error_to_status(int zlib_err) {
    switch (zlib_err) {
        case Z_OK:
        case Z_STREAM_END:
            return COMPRESS_OK;
        case Z_MEM_ERROR:
            return COMPRESS_ERR_MEMORY;
        case Z_DATA_ERROR:
            return COMPRESS_ERR_DATA;
        case Z_STREAM_ERROR:
            return COMPRESS_ERR_STREAM;
        case Z_VERSION_ERROR:
            return COMPRESS_ERR_VERSION;
        case Z_BUF_ERROR:
            return COMPRESS_ERR_BUFFER;
        default:
            return COMPRESS_ERR_INIT;
    }
}

// ============================================
// Compressor Lifecycle
// ============================================

// Create a new compressor.
// mode: COMPRESS_DEFLATE (0) or COMPRESS_GZIP (1)
// level: 1 (fast) to 9 (best compression)
// Returns opaque pointer to compressor_t, or 0 on failure.
long __compress_new(long mode, long level) {
    if (level < 1 || level > 9) {
        return 0;
    }

    compressor_t* comp = (compressor_t*)calloc(1, sizeof(compressor_t));
    if (comp == NULL) {
        return 0;
    }

    comp->mode = (int)mode;
    comp->level = (int)level;
    comp->initialized = 0;
    comp->streaming = 0;

    // Initialize z_stream
    comp->stream.zalloc = Z_NULL;
    comp->stream.zfree = Z_NULL;
    comp->stream.opaque = Z_NULL;

    return (long)comp;
}

// Free a compressor and its resources.
long __compress_free(long handle) {
    if (handle == 0) return COMPRESS_ERR_PARAM;

    compressor_t* comp = (compressor_t*)handle;

    if (comp->initialized) {
        if (comp->streaming) {
            deflateEnd(&comp->stream);
        } else {
            // Could be either deflate or inflate
            deflateEnd(&comp->stream);
            inflateEnd(&comp->stream);
        }
    }

    free(comp);
    return COMPRESS_OK;
}

// ============================================
// One-Shot Compression
// ============================================

// Compress data in one shot.
// data_ptr: pointer to input data
// data_len: length of input data
// out_ptr: pointer to receive output pointer (allocated by this function)
// out_len: pointer to receive output length
// Returns COMPRESS_OK on success, or error code.
long __compress_data(long handle, long data_ptr, long data_len,
                     long out_ptr, long out_len) {
    if (handle == 0 || data_ptr == 0 || data_len <= 0) {
        return COMPRESS_ERR_PARAM;
    }

    compressor_t* comp = (compressor_t*)handle;
    const unsigned char* input = (const unsigned char*)data_ptr;
    size_t input_len = (size_t)data_len;

    // Allocate output buffer (compressed data is usually smaller)
    size_t output_size = input_len + (input_len / 1000) + 64;  // Add overhead
    unsigned char* output = (unsigned char*)malloc(output_size);
    if (output == NULL) {
        return COMPRESS_ERR_MEMORY;
    }

    // Initialize z_stream for compression
    z_stream stream;
    stream.zalloc = Z_NULL;
    stream.zfree = Z_NULL;
    stream.opaque = Z_NULL;

    int ret;
    if (comp->mode == COMPRESS_GZIP) {
        // gzip format (RFC 1952) - use windowBits = 15 + 16
        ret = deflateInit2(&stream, comp->level, Z_DEFLATED,
                          15 + 16, 8, Z_DEFAULT_STRATEGY);
    } else {
        // raw deflate (RFC 1951) - use negative windowBits
        ret = deflateInit2(&stream, comp->level, Z_DEFLATED,
                          -15, 8, Z_DEFAULT_STRATEGY);
    }

    if (ret != Z_OK) {
        free(output);
        return zlib_error_to_status(ret);
    }

    stream.next_in = (unsigned char*)input;
    stream.avail_in = (uInt)input_len;
    stream.next_out = output;
    stream.avail_out = (uInt)output_size;

    // Compress in one shot
    ret = deflate(&stream, Z_FINISH);
    deflateEnd(&stream);

    if (ret != Z_STREAM_END) {
        free(output);
        return zlib_error_to_status(ret);
    }

    // Return output pointer and length
    size_t final_size = stream.total_out;
    long* out_ptr_ptr = (long*)out_ptr;
    long* out_len_ptr = (long*)out_len;

    *out_ptr_ptr = (long)output;
    *out_len_ptr = (long)final_size;

    return COMPRESS_OK;
}

// ============================================
// One-Shot Decompression
// ============================================

// Decompress data in one shot.
// data_ptr: pointer to compressed input data
// data_len: length of compressed input data
// out_ptr: pointer to receive output pointer (allocated by this function)
// out_len: pointer to receive output length
// Returns COMPRESS_OK on success, or error code.
long __decompress_data(long handle, long data_ptr, long data_len,
                       long out_ptr, long out_len) {
    if (handle == 0 || data_ptr == 0 || data_len <= 0) {
        return COMPRESS_ERR_PARAM;
    }

    compressor_t* comp = (compressor_t*)handle;
    const unsigned char* input = (const unsigned char*)data_ptr;
    size_t input_len = (size_t)data_len;

    // Allocate initial output buffer (decompressed data is usually larger)
    size_t output_size = input_len * 10;  // Assume 10x expansion max
    if (output_size < 4096) output_size = 4096;
    unsigned char* output = (unsigned char*)malloc(output_size);
    if (output == NULL) {
        return COMPRESS_ERR_MEMORY;
    }

    // Initialize z_stream for decompression
    z_stream stream;
    stream.zalloc = Z_NULL;
    stream.zfree = Z_NULL;
    stream.opaque = Z_NULL;
    stream.avail_in = 0;
    stream.next_in = Z_NULL;

    int ret;
    if (comp->mode == COMPRESS_GZIP) {
        // gzip format - use windowBits = 15 + 16
        ret = inflateInit2(&stream, 15 + 16);
    } else {
        // raw deflate - use negative windowBits
        ret = inflateInit2(&stream, -15);
    }

    if (ret != Z_OK) {
        free(output);
        return zlib_error_to_status(ret);
    }

    stream.next_in = (unsigned char*)input;
    stream.avail_in = (uInt)input_len;
    stream.next_out = output;
    stream.avail_out = (uInt)output_size;

    // Decompress
    ret = inflate(&stream, Z_FINISH);

    if (ret == Z_BUF_ERROR) {
        // Need larger buffer - reallocate and try again
        output_size = input_len * 50;  // Much larger expansion
        unsigned char* new_output = (unsigned char*)realloc(output, output_size);
        if (new_output == NULL) {
            inflateEnd(&stream);
            free(output);
            return COMPRESS_ERR_MEMORY;
        }
        output = new_output;
        stream.next_out = output + stream.total_out;
        stream.avail_out = (uInt)(output_size - stream.total_out);
        ret = inflate(&stream, Z_FINISH);
    }

    inflateEnd(&stream);

    if (ret != Z_STREAM_END) {
        free(output);
        return zlib_error_to_status(ret);
    }

    // Return output pointer and length
    size_t final_size = stream.total_out;
    long* out_ptr_ptr = (long*)out_ptr;
    long* out_len_ptr = (long*)out_len;

    *out_ptr_ptr = (long)output;
    *out_len_ptr = (long)final_size;

    return COMPRESS_OK;
}

// ============================================
// Streaming Compression
// ============================================

// Begin streaming compression.
long __compress_stream_begin(long handle) {
    if (handle == 0) return COMPRESS_ERR_PARAM;

    compressor_t* comp = (compressor_t*)handle;

    // Initialize z_stream for compression
    int ret;
    if (comp->mode == COMPRESS_GZIP) {
        // gzip format (RFC 1952)
        ret = deflateInit2(&comp->stream, comp->level, Z_DEFLATED,
                          15 + 16, 8, Z_DEFAULT_STRATEGY);
    } else {
        // raw deflate (RFC 1951)
        ret = deflateInit2(&comp->stream, comp->level, Z_DEFLATED,
                          -15, 8, Z_DEFAULT_STRATEGY);
    }

    if (ret != Z_OK) {
        return zlib_error_to_status(ret);
    }

    comp->initialized = 1;
    comp->streaming = 1;

    return COMPRESS_OK;
}

// Write a chunk to the compression stream.
// Returns compressed output (may be empty if data is buffered).
long __compress_stream_write(long handle, long chunk_ptr, long chunk_len,
                             long out_ptr, long out_len) {
    if (handle == 0 || !chunk_ptr || chunk_len <= 0) {
        return COMPRESS_ERR_PARAM;
    }

    compressor_t* comp = (compressor_t*)handle;
    if (!comp->initialized || !comp->streaming) {
        return COMPRESS_ERR_STREAM;
    }

    const unsigned char* input = (const unsigned char*)chunk_ptr;
    size_t input_len = (size_t)chunk_len;

    // Allocate output buffer
    size_t output_size = input_len + 1024;  // Add overhead for chunk
    unsigned char* output = (unsigned char*)malloc(output_size);
    if (output == NULL) {
        return COMPRESS_ERR_MEMORY;
    }

    comp->stream.next_in = (unsigned char*)input;
    comp->stream.avail_in = (uInt)input_len;
    comp->stream.next_out = output;
    comp->stream.avail_out = (uInt)output_size;

    // Compress this chunk (Z_NO_FLUSH to buffer data efficiently)
    int ret = deflate(&comp->stream, Z_NO_FLUSH);

    if (ret != Z_OK) {
        free(output);
        return zlib_error_to_status(ret);
    }

    // Calculate how much was output
    size_t produced = output_size - comp->stream.avail_out;

    // Return output (may be zero if data is buffered)
    long* out_ptr_ptr = (long*)out_ptr;
    long* out_len_ptr = (long*)out_len;

    if (produced > 0) {
        // Shrink buffer to actual size
        unsigned char* final_output = (unsigned char*)realloc(output, produced);
        if (final_output == NULL) final_output = output;  // Use original if realloc fails

        *out_ptr_ptr = (long)final_output;
        *out_len_ptr = (long)produced;
    } else {
        // No output produced yet
        free(output);
        *out_ptr_ptr = 0;
        *out_len_ptr = 0;
    }

    return COMPRESS_OK;
}

// Finish streaming compression and get final output.
long __compress_stream_finish(long handle, long out_ptr, long out_len) {
    if (handle == 0) return COMPRESS_ERR_PARAM;

    compressor_t* comp = (compressor_t*)handle;
    if (!comp->initialized || !comp->streaming) {
        return COMPRESS_ERR_STREAM;
    }

    // Allocate output buffer for final flush
    size_t output_size = COMPRESS_CHUNK_SIZE;
    unsigned char* output = (unsigned char*)malloc(output_size);
    if (output == NULL) {
        return COMPRESS_ERR_MEMORY;
    }

    comp->stream.next_in = Z_NULL;
    comp->stream.avail_in = 0;
    comp->stream.next_out = output;
    comp->stream.avail_out = (uInt)output_size;

    // Finish compression (Z_FINISH flushes all buffered data)
    int ret = deflate(&comp->stream, Z_FINISH);

    deflateEnd(&comp->stream);
    comp->initialized = 0;
    comp->streaming = 0;

    if (ret != Z_STREAM_END) {
        free(output);
        return zlib_error_to_status(ret);
    }

    // Calculate final output size
    size_t produced = output_size - comp->stream.avail_out;

    // Return output
    long* out_ptr_ptr = (long*)out_ptr;
    long* out_len_ptr = (long*)out_len;

    if (produced > 0) {
        // Shrink buffer to actual size
        unsigned char* final_output = (unsigned char*)realloc(output, produced);
        if (final_output == NULL) final_output = output;

        *out_ptr_ptr = (long)final_output;
        *out_len_ptr = (long)produced;
    } else {
        free(output);
        *out_ptr_ptr = 0;
        *out_len_ptr = 0;
    }

    return COMPRESS_OK;
}
