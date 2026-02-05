// Vais Runtime - Basic memory operations
#include <stdint.h>
#include <stdlib.h>

int64_t load_i64(int64_t ptr) {
    return *((int64_t*)ptr);
}

int64_t store_i64(int64_t ptr, int64_t value) {
    *((int64_t*)ptr) = value;
    return 0;
}

// GC stubs (selfhost compiler doesn't need GC)
int64_t vais_gc_alloc(int64_t size, int32_t type_id) {
    return (int64_t)malloc(size);
}

int64_t vais_gc_init() { return 0; }
int64_t vais_gc_collect() { return 0; }
int64_t vais_gc_add_root(int64_t ptr) { return 0; }
int64_t vais_gc_remove_root(int64_t ptr) { return 0; }
int64_t vais_gc_set_threshold(int64_t bytes) { return 0; }
int64_t vais_gc_print_stats() { return 0; }
int64_t vais_gc_bytes_allocated() { return 0; }
int64_t vais_gc_objects_count() { return 0; }
int64_t vais_gc_collections() { return 0; }
