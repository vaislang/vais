# Channel

Inter-thread communication via message passing.

**Module:** `std/channel.vais`

## Types

### `UnboundedChannel<T>`

An unbounded multi-producer, single-consumer channel.

```vais
S UnboundedChannel<T> {
    # internal implementation
}
```

### `ChannelSet`

A selector for waiting on multiple channels simultaneously.

## Functions

### `channel_send(ch: UnboundedChannel<T>, val: T)`

Sends a value into the channel. Never blocks.

```vais
ch := UnboundedChannel<i64> {}
channel_send(ch, 42)
```

### `channel_recv(ch: UnboundedChannel<T>) -> T`

Receives a value from the channel. Blocks until a value is available.

```vais
val := channel_recv(ch)
```

### `channel_try_recv(ch: UnboundedChannel<T>) -> Option<T>`

Attempts to receive without blocking. Returns `None` if empty.

```vais
M channel_try_recv(ch) {
    Some(val) => puts("got {val}"),
    None => puts("empty"),
}
```

### `channel_close(ch: UnboundedChannel<T>)`

Closes the channel. Subsequent sends will fail.

## ChannelSet Select

Wait on multiple channels:

```vais
cs := ChannelSet {}
cs.add(ch1)
cs.add(ch2)
idx := cs.select()  # returns index of ready channel
```

## See Also

- [Sync](./sync.md) — synchronization primitives (Mutex, RwLock, Channel)
- [Thread](./thread.md) — thread creation
- [Async](./async.md) — async channel alternatives
