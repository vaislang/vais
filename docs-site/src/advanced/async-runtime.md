# Async Runtime

Vais provides a comprehensive async runtime with Future-based concurrency, enabling efficient asynchronous I/O and task scheduling.

## Core Concepts

### Futures

Futures represent asynchronous computations that will complete in the future:

```vais
U std/async

F fetch_data() -> Future<str> {
    # Async computation
    async_http_get("https://api.example.com/data")
}

F main() -> i64 {
    future := fetch_data()
    result := Y future  # Await the future
    puts("Got: {result}")
    0
}
```

The `Y` operator (await) suspends execution until the future completes.

## Task Spawning

### spawn

Create concurrent tasks that run independently:

```vais
U std/async

F background_task(id: i64) -> Future<()> {
    sleep(1000)
    puts("Task {id} completed")
}

F main() -> i64 {
    spawn(background_task(1))
    spawn(background_task(2))
    spawn(background_task(3))

    # Wait for all tasks
    sleep(2000)
    0
}
```

### select

Wait for the first of multiple futures to complete:

```vais
U std/async

F main() -> i64 {
    future_a := fetch_data_a()
    future_b := fetch_data_b()

    result := select([future_a, future_b])
    puts("First result: {result}")
    0
}
```

### join

Wait for all futures to complete:

```vais
U std/async

F main() -> i64 {
    futures := [
        fetch_user(1),
        fetch_user(2),
        fetch_user(3)
    ]

    results := join(futures)
    # Process all results
    0
}
```

## Future Combinators

### map / flat_map

Transform future values:

```vais
U std/async

F main() -> i64 {
    future := fetch_number()
        .map(|n| n * 2)
        .flat_map(|n| fetch_string(n))

    result := Y future
    0
}
```

### filter

Filter future values:

```vais
future := fetch_users()
    .filter(|user| user.age >= 18)
```

### race

Return the first future to complete (similar to `select` but with different semantics):

```vais
winner := race([slow_fetch(), fast_fetch()])
```

### retry

Retry failed futures:

```vais
result := fetch_with_retry()
    .retry(3)  # Retry up to 3 times
```

### chain

Chain futures sequentially:

```vais
result := fetch_token()
    .chain(|token| fetch_user(token))
    .chain(|user| fetch_posts(user.id))
```

### fuse

Prevent multiple polls after completion:

```vais
future := fetch_data().fuse()
```

## Synchronization Primitives

### Barrier

Coordinate multiple tasks to reach a synchronization point:

```vais
U std/async

F main() -> i64 {
    barrier := Barrier::new(3)

    spawn(worker(barrier, 1))
    spawn(worker(barrier, 2))
    spawn(worker(barrier, 3))

    # All workers will wait at the barrier
    0
}

F worker(barrier: Barrier, id: i64) -> Future<()> {
    puts("Worker {id} starting")
    Y barrier.wait()
    puts("Worker {id} past barrier")
}
```

### Semaphore

Limit concurrent access to a resource:

```vais
U std/async

F main() -> i64 {
    sem := Semaphore::new(2)  # Allow 2 concurrent tasks

    spawn(limited_task(sem, 1))
    spawn(limited_task(sem, 2))
    spawn(limited_task(sem, 3))  # Will wait for slot

    0
}

F limited_task(sem: Semaphore, id: i64) -> Future<()> {
    Y sem.acquire()
    puts("Task {id} running")
    sleep(1000)
    sem.release()
}
```

### WaitGroup

Wait for a group of tasks to complete:

```vais
U std/async

F main() -> i64 {
    wg := WaitGroup::new()

    wg.add(3)
    spawn(task(wg, 1))
    spawn(task(wg, 2))
    spawn(task(wg, 3))

    Y wg.wait()  # Wait for all tasks
    puts("All tasks completed")
    0
}

F task(wg: WaitGroup, id: i64) -> Future<()> {
    sleep(id * 100)
    puts("Task {id} done")
    wg.done()
}
```

### OnceCell

Initialize a value exactly once in a concurrent context:

```vais
U std/async

global config: OnceCell<Config> = OnceCell::new()

F get_config() -> Future<Config> {
    Y config.get_or_init(|| load_config())
}
```

### AsyncStream

Stream values asynchronously:

```vais
U std/async

F generate_numbers() -> AsyncStream<i64> {
    stream := AsyncStream::new()

    L i := 0; i < 10; i = i + 1 {
        Y stream.send(i)
        Y sleep(100)
    }

    stream
}

F main() -> i64 {
    stream := generate_numbers()

    L {
        M Y stream.next() {
            Some(n) => puts("Got: {n}"),
            None => B
        }
    }

    0
}
```

## Async I/O

### File I/O

```vais
U std/async_io

F main() -> i64 {
    content := Y async_read_file("input.txt")
    Y async_write_file("output.txt", content)
    0
}
```

### Network I/O

```vais
U std/async_net

F main() -> i64 {
    listener := Y AsyncTcpListener::bind("127.0.0.1:8080")

    L {
        stream := Y listener.accept()
        spawn(handle_connection(stream))
    }

    0
}

F handle_connection(stream: AsyncTcpStream) -> Future<()> {
    buffer := [0u8; 1024]
    n := Y stream.read(buffer)
    Y stream.write(buffer[0..n])
}
```

### HTTP Server

```vais
U std/async_http

F main() -> i64 {
    server := AsyncHttpServer::new("127.0.0.1:8080")

    server.get("/", |req| {
        AsyncHttpResponse {
            status: 200,
            body: "Hello, World!"
        }
    })

    Y server.run()
}
```

### HTTP Client

```vais
U std/async_http

F main() -> i64 {
    client := AsyncHttpClient::new()
    response := Y client.get("https://api.example.com/data")

    M response.status {
        200 => {
            body := Y response.text()
            puts("Success: {body}")
        },
        _ => puts("Error: {response.status}")
    }

    0
}
```

## Runtime Configuration

The async runtime can be configured:

```vais
U std/async

F main() -> i64 {
    runtime := AsyncRuntime::new()
        .worker_threads(4)
        .max_blocking_threads(16)
        .thread_name("vais-worker")

    runtime.block_on(async_main())
}

F async_main() -> Future<i64> {
    # Your async code here
    0
}
```

## Standard Library Modules

- **std/async.vais** — Core async primitives (Future, spawn, select, join)
- **std/async_io.vais** — Async file I/O operations
- **std/async_net.vais** — Async networking (TCP, UDP)
- **std/async_http.vais** — Async HTTP client and server

## Performance Tips

1. **Batch operations** — Use `join` to parallelize independent tasks
2. **Limit concurrency** — Use `Semaphore` to prevent resource exhaustion
3. **Stream processing** — Use `AsyncStream` for large datasets
4. **Avoid blocking** — Never block in async tasks (use async I/O)

## Error Handling

Combine async with Result types:

```vais
U std/async

F fetch_data() -> Future<Result<str, Error>> {
    # May fail
    async_http_get("https://api.example.com/data")
}

F main() -> i64 {
    M Y fetch_data() {
        Ok(data) => puts("Success: {data}"),
        Err(e) => puts("Error: {e}")
    }
    0
}
```

## See Also

- [Async Tutorial](../language/async-tutorial.md) — Getting started with async programming
- [Future API](../api/future.md) — Future type reference
- [Async Reactor](../api/async_reactor.md) — Low-level reactor API
- [Runtime](../api/runtime.md) — Runtime configuration
