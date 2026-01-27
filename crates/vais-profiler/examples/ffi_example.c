/*
 * Example of using vais-profiler from C
 *
 * Compile with:
 * gcc -o ffi_example ffi_example.c -L../target/release -lvais_profiler
 */

#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <unistd.h>

// FFI declarations
typedef struct {
    uint64_t sample_interval_ms;
    bool track_memory;
    bool build_call_graph;
    size_t max_samples;
} VaisProfilerConfig;

typedef struct {
    size_t sample_count;
    size_t total_allocations;
    size_t total_allocated_bytes;
    size_t current_allocated_bytes;
    size_t peak_allocated_bytes;
    size_t call_graph_edges;
} VaisProfileStats;

extern void* vais_profiler_create(const VaisProfilerConfig* config);
extern void vais_profiler_destroy(void* profiler);
extern bool vais_profiler_start(void* profiler);
extern bool vais_profiler_stop(void* profiler);
extern bool vais_profiler_is_running(void* profiler);
extern void vais_profiler_record_sample(void* profiler, const char* function_name, size_t ip);
extern void vais_profiler_record_allocation(void* profiler, size_t size, size_t address);
extern void vais_profiler_record_deallocation(void* profiler, size_t address);
extern void vais_profiler_record_call(void* profiler, const char* caller, const char* callee);
extern VaisProfileStats vais_profiler_get_stats(void* profiler);

// Simulated workload
void compute_intensive(void* profiler) {
    for (int i = 0; i < 1000; i++) {
        vais_profiler_record_sample(profiler, "compute_intensive", 0x2000 + i);

        // Simulate some allocations
        if (i % 10 == 0) {
            vais_profiler_record_allocation(profiler, 1024, 0x10000 + i * 1024);
        }
    }
}

void process_data(void* profiler) {
    for (int i = 0; i < 500; i++) {
        vais_profiler_record_sample(profiler, "process_data", 0x3000 + i);
    }
}

int main() {
    printf("=== Vais Profiler C Example ===\n\n");

    // Create profiler with default config
    void* profiler = vais_profiler_create(NULL);
    if (profiler == NULL) {
        fprintf(stderr, "Failed to create profiler\n");
        return 1;
    }

    // Start profiling
    printf("Starting profiler...\n");
    if (!vais_profiler_start(profiler)) {
        fprintf(stderr, "Failed to start profiler\n");
        vais_profiler_destroy(profiler);
        return 1;
    }

    // Main loop
    for (int i = 0; i < 100; i++) {
        vais_profiler_record_sample(profiler, "main", 0x1000 + i);
    }

    // Record function calls
    vais_profiler_record_call(profiler, "main", "compute_intensive");
    compute_intensive(profiler);

    vais_profiler_record_call(profiler, "main", "process_data");
    process_data(profiler);

    // Allocate some memory
    for (int i = 0; i < 50; i++) {
        vais_profiler_record_allocation(profiler, 2048, 0x20000 + i * 2048);
    }

    // Deallocate some memory
    for (int i = 0; i < 25; i++) {
        vais_profiler_record_deallocation(profiler, 0x20000 + i * 2048);
    }

    usleep(100000); // Sleep for 100ms

    // Stop profiling
    printf("Stopping profiler...\n\n");
    vais_profiler_stop(profiler);

    // Get and display statistics
    VaisProfileStats stats = vais_profiler_get_stats(profiler);

    printf("=== Profile Results ===\n");
    printf("Total samples:          %zu\n", stats.sample_count);
    printf("Total allocations:      %zu\n", stats.total_allocations);
    printf("Total allocated:        %zu bytes (%.2f MB)\n",
           stats.total_allocated_bytes,
           stats.total_allocated_bytes / 1048576.0);
    printf("Current allocated:      %zu bytes (%.2f MB)\n",
           stats.current_allocated_bytes,
           stats.current_allocated_bytes / 1048576.0);
    printf("Peak allocated:         %zu bytes (%.2f MB)\n",
           stats.peak_allocated_bytes,
           stats.peak_allocated_bytes / 1048576.0);
    printf("Call graph edges:       %zu\n", stats.call_graph_edges);

    // Cleanup
    vais_profiler_destroy(profiler);

    printf("\nProfiler destroyed successfully.\n");
    return 0;
}
