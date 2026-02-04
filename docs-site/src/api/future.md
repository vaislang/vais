# Future API Reference

> Stackless coroutine-based async/await with future combinators

## Import

```vais
U std/future
```

## Types

### Poll

```vais
E Poll { Pending, Ready(i64) }
```

| Method | Description |
|--------|-------------|
| `is_ready(&self) -> i64` | Returns 1 if Ready |
| `is_pending(&self) -> i64` | Returns 1 if Pending |
| `unwrap(&self) -> i64` | Get value (0 if Pending) |

### Future (trait)

```vais
W Future { F poll(&self, ctx: i64) -> Poll }
```

## Combinator Structs

| Struct | Description |
|--------|-------------|
| `MapFuture` | Transforms the output of a future |
| `AndThenFuture` | Chains futures sequentially |
| `JoinFuture` | Runs two futures concurrently |
| `SelectFuture` | Returns when either future completes |
| `ReadyFuture` | Immediately resolves to a value |
| `PendingFuture` | Never resolves |
| `TimerFuture` | Completes after N iterations |
| `YieldNow` | Yields control to the runtime |

## Helper Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `ready` | `F ready(value: i64) -> ReadyFuture` | Create immediately-ready future |
| `pending` | `F pending() -> PendingFuture` | Create never-resolving future |
| `join` | `F join(f1_ptr, f1_poll, f2_ptr, f2_poll) -> JoinFuture` | Join two futures |
| `select` | `F select(f1_ptr, f1_poll, f2_ptr, f2_poll) -> SelectFuture` | Race two futures |
| `delay` | `F delay(ticks: i64) -> TimerFuture` | Create timer future |
| `yield_now` | `F yield_now() -> YieldNow` | Create yield point |

## Usage

```vais
U std/future

F main() -> i64 {
    f := ready(42)
    # In async context: result := f.poll(ctx)
    0
}
```
