//! Memory profiling benchmarks for Vais compiler
//!
//! Measures memory allocation and peak usage during compilation stages.
//! Uses a custom global allocator wrapper to track memory statistics.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs;

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Memory tracking allocator
struct MemoryTracker {
    allocated: AtomicUsize,
    deallocated: AtomicUsize,
    peak: AtomicUsize,
}

impl MemoryTracker {
    const fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            deallocated: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }
    }

    fn record_alloc(&self, size: usize) {
        let prev = self.allocated.fetch_add(size, Ordering::Relaxed);
        let current = prev + size;
        let deallocated = self.deallocated.load(Ordering::Relaxed);
        let net = current.saturating_sub(deallocated);

        // Update peak if necessary
        let mut peak = self.peak.load(Ordering::Relaxed);
        while net > peak {
            match self.peak.compare_exchange(peak, net, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(current_peak) => peak = current_peak,
            }
        }
    }

    fn record_dealloc(&self, size: usize) {
        self.deallocated.fetch_add(size, Ordering::Relaxed);
    }

    fn current_usage(&self) -> usize {
        let allocated = self.allocated.load(Ordering::Relaxed);
        let deallocated = self.deallocated.load(Ordering::Relaxed);
        allocated.saturating_sub(deallocated)
    }

    fn peak_usage(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }

    fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.deallocated.store(0, Ordering::Relaxed);
        self.peak.store(0, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for MemoryTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            self.record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.record_dealloc(layout.size());
    }
}

#[global_allocator]
static MEMORY_TRACKER: MemoryTracker = MemoryTracker::new();

/// Memory statistics
#[derive(Debug, Clone, Copy)]
struct MemoryStats {
    peak_bytes: usize,
    current_bytes: usize,
}

impl MemoryStats {
    fn capture() -> Self {
        Self {
            peak_bytes: MEMORY_TRACKER.peak_usage(),
            current_bytes: MEMORY_TRACKER.current_usage(),
        }
    }

    fn format_bytes(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = 1024 * KB;
        const GB: usize = 1024 * MB;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }

    fn peak_formatted(&self) -> String {
        Self::format_bytes(self.peak_bytes)
    }

    fn current_formatted(&self) -> String {
        Self::format_bytes(self.current_bytes)
    }
}

/// Load a fixture file
fn load_fixture(name: &str) -> String {
    // Try relative path first (when running from project root)
    let relative_path = format!("benches/fixtures/{}.vais", name);
    if let Ok(content) = fs::read_to_string(&relative_path) {
        return content;
    }

    // Try path relative to benches directory
    let bench_relative = format!("fixtures/{}.vais", name);
    if let Ok(content) = fs::read_to_string(&bench_relative) {
        return content;
    }

    // Try using CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = format!("{}/benches/fixtures/{}.vais", manifest_dir, name);
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            return content;
        }
    }

    panic!("Failed to load fixture: {}", name)
}

/// Generate synthetic Vais code with N functions
fn generate_code(num_funcs: usize) -> String {
    let mut code = String::new();

    for i in 0..num_funcs {
        code.push_str(&format!(
            "F func{}(x: i64)->i64 = x * {} + {}\n",
            i,
            i % 10,
            i
        ));
    }

    code.push_str("F main()->i64 = func0(42)\n");
    code
}

/// Estimate lines of code
fn estimate_lines(num_funcs: usize) -> usize {
    num_funcs + 1 // +1 for main
}

/// Memory benchmark: Lexer
fn bench_memory_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_lexer");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);

        group.bench_with_input(
            BenchmarkId::new("tokenize", fixture),
            &source,
            |b, s| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::ZERO;
                    let mut peak_memory = 0;

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start = std::time::Instant::now();
                        let _ = tokenize(black_box(s));
                        total_duration += start.elapsed();
                        peak_memory = peak_memory.max(MEMORY_TRACKER.peak_usage());
                    }

                    // Print memory stats (only on last iteration)
                    if iters > 0 {
                        eprintln!(
                            "[{}] lexer peak: {}",
                            fixture,
                            MemoryStats::format_bytes(peak_memory)
                        );
                    }

                    total_duration
                })
            },
        );
    }

    group.finish();
}

/// Memory benchmark: Parser
fn bench_memory_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_parser");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);

        group.bench_with_input(
            BenchmarkId::new("parse", fixture),
            &source,
            |b, s| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::ZERO;
                    let mut peak_memory = 0;

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start = std::time::Instant::now();
                        let _ = parse(black_box(s));
                        total_duration += start.elapsed();
                        peak_memory = peak_memory.max(MEMORY_TRACKER.peak_usage());
                    }

                    if iters > 0 {
                        eprintln!(
                            "[{}] parser peak: {}",
                            fixture,
                            MemoryStats::format_bytes(peak_memory)
                        );
                    }

                    total_duration
                })
            },
        );
    }

    group.finish();
}

/// Memory benchmark: Type checker
fn bench_memory_typechecker(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_typechecker");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);
        let ast = parse(&source).expect("Parse failed");

        group.bench_with_input(
            BenchmarkId::new("check", fixture),
            &ast,
            |b, ast| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::ZERO;
                    let mut peak_memory = 0;

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start = std::time::Instant::now();
                        let mut checker = TypeChecker::new();
                        let _ = checker.check_module(black_box(ast));
                        total_duration += start.elapsed();
                        peak_memory = peak_memory.max(MEMORY_TRACKER.peak_usage());
                    }

                    if iters > 0 {
                        eprintln!(
                            "[{}] typechecker peak: {}",
                            fixture,
                            MemoryStats::format_bytes(peak_memory)
                        );
                    }

                    total_duration
                })
            },
        );
    }

    group.finish();
}

/// Memory benchmark: Code generator
fn bench_memory_codegen(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_codegen");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);
        let ast = parse(&source).expect("Parse failed");

        // Type check first (required for codegen)
        let mut checker = TypeChecker::new();
        checker.check_module(&ast).expect("Type check failed");

        group.bench_with_input(
            BenchmarkId::new("generate", fixture),
            &ast,
            |b, ast| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::ZERO;
                    let mut peak_memory = 0;

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start = std::time::Instant::now();
                        let mut codegen = CodeGenerator::new(fixture);
                        let _ = codegen.generate_module(black_box(ast));
                        total_duration += start.elapsed();
                        peak_memory = peak_memory.max(MEMORY_TRACKER.peak_usage());
                    }

                    if iters > 0 {
                        eprintln!(
                            "[{}] codegen peak: {}",
                            fixture,
                            MemoryStats::format_bytes(peak_memory)
                        );
                    }

                    total_duration
                })
            },
        );
    }

    group.finish();
}

/// Memory benchmark: Full compilation pipeline
fn bench_memory_full_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_full_compile");

    for fixture in ["fibonacci", "sort", "struct_heavy", "complex"] {
        let source = load_fixture(fixture);

        group.bench_with_input(
            BenchmarkId::new("compile", fixture),
            &source,
            |b, source| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::ZERO;
                    let mut peak_memory = 0;

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start = std::time::Instant::now();

                        // Full pipeline
                        let _tokens = tokenize(black_box(source)).expect("Lex failed");
                        let ast = parse(black_box(source)).expect("Parse failed");
                        let mut checker = TypeChecker::new();
                        checker.check_module(&ast).expect("Type check failed");
                        let mut codegen = CodeGenerator::new("bench");
                        let _ = codegen.generate_module(&ast);

                        total_duration += start.elapsed();
                        peak_memory = peak_memory.max(MEMORY_TRACKER.peak_usage());
                    }

                    if iters > 0 {
                        eprintln!(
                            "[{}] full compile peak: {}",
                            fixture,
                            MemoryStats::format_bytes(peak_memory)
                        );
                    }

                    total_duration
                })
            },
        );
    }

    group.finish();
}

/// Memory scaling benchmark: Varying input sizes
fn bench_memory_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_scaling");

    // Test with different code sizes (lines of code)
    let sizes = [
        (50, "1K"),      // ~50 funcs ≈ 1K lines
        (250, "5K"),     // ~250 funcs ≈ 5K lines
        (500, "10K"),    // ~500 funcs ≈ 10K lines
        (2500, "50K"),   // ~2500 funcs ≈ 50K lines
    ];

    for (num_funcs, label) in sizes {
        let source = generate_code(num_funcs);
        let lines = estimate_lines(num_funcs);

        group.bench_with_input(
            BenchmarkId::new("compile", label),
            &source,
            |b, s| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::ZERO;
                    let mut peak_memory = 0;

                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start = std::time::Instant::now();

                        // Full pipeline
                        let _tokens = tokenize(black_box(s)).expect("Lex failed");
                        let ast = parse(black_box(s)).expect("Parse failed");
                        let mut checker = TypeChecker::new();
                        checker.check_module(&ast).expect("Type check failed");
                        let mut codegen = CodeGenerator::new("bench");
                        let _ = codegen.generate_module(&ast);

                        total_duration += start.elapsed();
                        peak_memory = peak_memory.max(MEMORY_TRACKER.peak_usage());
                    }

                    if iters > 0 {
                        let kb_per_line = peak_memory as f64 / 1024.0 / lines as f64;
                        eprintln!(
                            "[{}] {} lines → peak: {} ({:.2} KB/line)",
                            label,
                            lines,
                            MemoryStats::format_bytes(peak_memory),
                            kb_per_line
                        );
                    }

                    total_duration
                })
            },
        );
    }

    group.finish();
}

/// Memory benchmark: Per-stage breakdown
fn bench_memory_stages(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_stages");

    let source = load_fixture("complex");

    group.bench_function("stage_breakdown", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::ZERO;

            for _ in 0..iters {
                MEMORY_TRACKER.reset();

                // Stage 1: Lex
                let start = std::time::Instant::now();
                let _tokens = tokenize(black_box(&source)).expect("Lex failed");
                let lex_duration = start.elapsed();
                let lex_peak = MEMORY_TRACKER.peak_usage();

                // Stage 2: Parse
                MEMORY_TRACKER.reset();
                let start = std::time::Instant::now();
                let ast = parse(black_box(&source)).expect("Parse failed");
                let parse_duration = start.elapsed();
                let parse_peak = MEMORY_TRACKER.peak_usage();

                // Stage 3: Type check
                MEMORY_TRACKER.reset();
                let start = std::time::Instant::now();
                let mut checker = TypeChecker::new();
                checker.check_module(&ast).expect("Type check failed");
                let typecheck_duration = start.elapsed();
                let typecheck_peak = MEMORY_TRACKER.peak_usage();

                // Stage 4: Codegen
                MEMORY_TRACKER.reset();
                let start = std::time::Instant::now();
                let mut codegen = CodeGenerator::new("bench");
                let _ = codegen.generate_module(&ast);
                let codegen_duration = start.elapsed();
                let codegen_peak = MEMORY_TRACKER.peak_usage();

                total_duration += lex_duration + parse_duration + typecheck_duration + codegen_duration;

                // Print detailed breakdown (only on last iteration)
                if iters > 0 {
                    eprintln!("\n=== Memory Stage Breakdown ===");
                    eprintln!("  Lex:       {} ({:?})", MemoryStats::format_bytes(lex_peak), lex_duration);
                    eprintln!("  Parse:     {} ({:?})", MemoryStats::format_bytes(parse_peak), parse_duration);
                    eprintln!("  Typecheck: {} ({:?})", MemoryStats::format_bytes(typecheck_peak), typecheck_duration);
                    eprintln!("  Codegen:   {} ({:?})", MemoryStats::format_bytes(codegen_peak), codegen_duration);
                    eprintln!("==============================\n");
                }
            }

            total_duration
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_memory_lexer,
    bench_memory_parser,
    bench_memory_typechecker,
    bench_memory_codegen,
    bench_memory_full_compile,
    bench_memory_scaling,
    bench_memory_stages,
);

criterion_main!(benches);
