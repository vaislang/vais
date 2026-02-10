//! TCP 10K Concurrent Connection Benchmark
//!
//! Measures TCP server performance under various connection patterns:
//! - Connection creation rate
//! - Concurrent connection handling
//! - Data throughput
//! - Connection churn (rapid connect/disconnect)
//!
//! These benchmarks test the fundamental building blocks for handling
//! high-concurrency network services (C10K problem).
//!
//! Note: To enable this benchmark in Cargo.toml, add:
//! ```toml
//! [[bench]]
//! name = "tcp_bench"
//! harness = false
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Helper: Create a TCP listener on a random port
fn create_listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener")
}

/// Helper: Get the actual port a listener is bound to
fn get_listener_port(listener: &TcpListener) -> u16 {
    listener.local_addr().unwrap().port()
}

/// Benchmark: TCP connection creation rate
///
/// Measures how fast we can create TCP connections in batches.
/// Tests: 10, 100, 1000 connections
#[cfg(not(target_os = "windows"))]
fn bench_tcp_connections(c: &mut Criterion) {
    let mut group = c.benchmark_group("tcp_connections");
    group.sample_size(20); // Reduce samples for stability

    for batch_size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("create_batch", batch_size),
            &batch_size,
            |b, &count| {
                b.iter(|| {
                    // Create listener
                    let listener = create_listener();
                    let port = get_listener_port(&listener);

                    // Accept connections in background thread
                    let shutdown = Arc::new(AtomicBool::new(false));
                    let shutdown_clone = shutdown.clone();
                    let accept_thread = thread::spawn(move || {
                        listener.set_nonblocking(true).ok();
                        let mut accepted = 0;
                        while accepted < count && !shutdown_clone.load(Ordering::Relaxed) {
                            if let Ok((stream, _)) = listener.accept() {
                                drop(stream);
                                accepted += 1;
                            } else {
                                thread::sleep(Duration::from_micros(100));
                            }
                        }
                        accepted
                    });

                    // Create connections
                    let mut connections = Vec::new();
                    for _ in 0..count {
                        match TcpStream::connect(("127.0.0.1", port)) {
                            Ok(stream) => connections.push(stream),
                            Err(_) => break,
                        }
                    }

                    shutdown.store(true, Ordering::Relaxed);
                    let accepted = accept_thread.join().unwrap_or(0);

                    black_box(accepted);
                    drop(connections);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Concurrent connection handling
///
/// Measures the time to establish and hold multiple simultaneous connections.
/// Tests: 100, 1000, 5000 concurrent connections
#[cfg(not(target_os = "windows"))]
fn bench_concurrent_hold(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_hold");
    group.sample_size(10); // Fewer samples for large connection counts
    group.measurement_time(Duration::from_secs(20)); // More time for stability

    for conn_count in [100, 1000, 5000] {
        group.bench_with_input(
            BenchmarkId::new("hold_connections", conn_count),
            &conn_count,
            |b, &count| {
                b.iter(|| {
                    // Create listener with larger backlog
                    let listener = create_listener();
                    let port = get_listener_port(&listener);

                    // Accept connections in background
                    let shutdown = Arc::new(AtomicBool::new(false));
                    let shutdown_clone = shutdown.clone();
                    let accept_thread = thread::spawn(move || {
                        listener.set_nonblocking(true).ok();
                        let mut streams = Vec::new();
                        while streams.len() < count && !shutdown_clone.load(Ordering::Relaxed) {
                            match listener.accept() {
                                Ok((stream, _)) => {
                                    stream.set_nodelay(true).ok();
                                    streams.push(stream);
                                }
                                Err(_) => thread::sleep(Duration::from_micros(50)),
                            }
                        }
                        streams
                    });

                    // Create client connections
                    let mut connections = Vec::new();
                    for _ in 0..count {
                        match TcpStream::connect_timeout(
                            &format!("127.0.0.1:{}", port).parse().unwrap(),
                            Duration::from_secs(5),
                        ) {
                            Ok(stream) => {
                                stream.set_nodelay(true).ok();
                                stream.set_nonblocking(true).ok();
                                connections.push(stream);
                            }
                            Err(_) => break,
                        }
                    }

                    // Hold connections briefly
                    thread::sleep(Duration::from_millis(100));

                    shutdown.store(true, Ordering::Relaxed);
                    let server_streams = accept_thread.join().unwrap_or_default();

                    black_box((connections.len(), server_streams.len()));
                    drop(connections);
                    drop(server_streams);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: TCP throughput (echo pattern)
///
/// Measures data transfer rate with different message sizes.
/// Tests: 64B, 1KB, 64KB messages
#[cfg(not(target_os = "windows"))]
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("tcp_throughput");
    group.sample_size(20);

    for msg_size in [64, 1024, 65536] {
        group.bench_with_input(
            BenchmarkId::new("echo_bytes", msg_size),
            &msg_size,
            |b, &size| {
                b.iter(|| {
                    // Create listener
                    let listener = create_listener();
                    let port = get_listener_port(&listener);

                    // Echo server thread
                    let echo_thread = thread::spawn(move || {
                        if let Ok((mut stream, _)) = listener.accept() {
                            stream.set_nodelay(true).ok();
                            let mut buf = vec![0u8; size];
                            if let Ok(n) = stream.read(&mut buf) {
                                stream.write_all(&buf[..n]).ok();
                            }
                        }
                    });

                    // Client: connect, send, receive
                    if let Ok(mut client) = TcpStream::connect(("127.0.0.1", port)) {
                        client.set_nodelay(true).ok();
                        let send_data = vec![0xAAu8; size];
                        client.write_all(&send_data).ok();

                        let mut recv_buf = vec![0u8; size];
                        let mut total_read = 0;
                        while total_read < size {
                            if let Ok(n) = client.read(&mut recv_buf[total_read..]) {
                                if n == 0 {
                                    break;
                                }
                                total_read += n;
                            } else {
                                break;
                            }
                        }
                        black_box(total_read);
                    }

                    echo_thread.join().ok();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Connection churn (rapid connect/disconnect)
///
/// Measures average latency for rapid connection cycles.
/// Creates 1000 sequential connect-send-receive-close cycles.
#[cfg(not(target_os = "windows"))]
fn bench_connection_churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_churn");
    group.sample_size(10);

    group.bench_function("churn_1000_cycles", |b| {
        b.iter(|| {
            let listener = create_listener();
            let port = get_listener_port(&listener);

            // Background acceptor
            let shutdown = Arc::new(AtomicBool::new(false));
            let shutdown_clone = shutdown.clone();
            let accept_thread = thread::spawn(move || {
                listener.set_nonblocking(true).ok();
                let mut handled = 0;
                while handled < 1000 && !shutdown_clone.load(Ordering::Relaxed) {
                    if let Ok((mut stream, _)) = listener.accept() {
                        let mut buf = [0u8; 4];
                        if stream.read_exact(&mut buf).is_ok() {
                            stream.write_all(&buf).ok();
                        }
                        handled += 1;
                    } else {
                        thread::sleep(Duration::from_micros(50));
                    }
                }
                handled
            });

            // Client: 1000 rapid cycles
            let mut successful = 0;
            for _ in 0..1000 {
                if let Ok(mut client) = TcpStream::connect(("127.0.0.1", port)) {
                    client.set_nodelay(true).ok();
                    let data = [0xBB; 4];
                    if client.write_all(&data).is_ok() {
                        let mut recv = [0u8; 4];
                        if client.read_exact(&mut recv).is_ok() {
                            successful += 1;
                        }
                    }
                    drop(client);
                }
            }

            shutdown.store(true, Ordering::Relaxed);
            let handled = accept_thread.join().unwrap_or(0);

            black_box((successful, handled));
        });
    });

    group.finish();
}

// Windows stubs (TCP benchmarks require Unix features)
#[cfg(target_os = "windows")]
fn bench_tcp_connections(_c: &mut Criterion) {}

#[cfg(target_os = "windows")]
fn bench_concurrent_hold(_c: &mut Criterion) {}

#[cfg(target_os = "windows")]
fn bench_throughput(_c: &mut Criterion) {}

#[cfg(target_os = "windows")]
fn bench_connection_churn(_c: &mut Criterion) {}

criterion_group!(
    benches,
    bench_tcp_connections,
    bench_concurrent_hold,
    bench_throughput,
    bench_connection_churn
);

criterion_main!(benches);
