# cache

In-memory LRU (Least Recently Used) cache implementation for Vais programs.

## Features

- Fixed capacity with automatic eviction
- LRU eviction policy
- String key-value storage
- O(n) operations (simplified implementation)
- Clear and remove operations
- Size tracking

## Usage

```vais
U cache

F main() -> i64 {
    # Create cache with capacity of 10
    cache := Cache.new(10)

    # Put key-value pairs
    cache.put("user:1", "John")
    cache.put("user:2", "Jane")
    cache.put("user:3", "Bob")

    # Get values
    name := cache.get("user:1")
    I name != 0 {
        puts_ptr("User 1: ")
        puts_ptr(name)
    }

    # Check if key exists
    I cache.has("user:2") {
        puts_ptr("User 2 exists in cache")
    }

    # Remove key
    I cache.remove("user:3") {
        puts_ptr("User 3 removed")
    }

    # Check cache size
    printf("Cache size: %d/%d\n", cache.size(), cache.capacity)

    # Clear cache
    cleared := cache.clear()
    printf("Cleared %d entries\n", cleared)

    # Free cache
    cache.free()

    0
}
```

## API

### Cache Methods

- `Cache.new(capacity: i64) -> Cache` - Create cache with capacity
- `cache.get(key: i64) -> i64` - Get value by key (returns 0 if not found)
- `cache.put(key: i64, value: i64) -> i64` - Put key-value pair
- `cache.has(key: i64) -> i64` - Check if key exists
- `cache.remove(key: i64) -> i64` - Remove entry by key
- `cache.clear() -> i64` - Clear all entries (returns count cleared)
- `cache.free() -> i64` - Free cache and all entries
- `cache.size() -> i64` - Get current number of entries
- `cache.is_empty() -> i64` - Check if cache is empty
- `cache.is_full() -> i64` - Check if cache is full

### Cache Fields

- `cache.capacity` - Maximum number of entries
- `cache.count` - Current number of entries
- `cache.clock` - Internal LRU clock

## How LRU Works

When the cache is full and a new entry is added:
1. Find the least recently used entry (oldest access time)
2. Evict that entry
3. Add the new entry

Access time is updated on:
- `get()` - Reading a value
- `put()` - Adding or updating a value

## Examples

### Simple Cache

```vais
cache := Cache.new(3)

cache.put("a", "1")
cache.put("b", "2")
cache.put("c", "3")

# Cache is full (3/3)
# Add new entry - will evict "a" (LRU)
cache.put("d", "4")

# Now cache contains: b, c, d
I cache.has("a") == 0 {
    puts_ptr("Entry 'a' was evicted")
}
```

### Database Query Cache

```vais
F get_user(cache: Cache, user_id: i64) -> i64 {
    key := malloc(64)
    sprintf(key, "user:%d", user_id)

    # Check cache first
    cached := cache.get(key)
    I cached != 0 {
        free(key)
        R cached  # Cache hit
    }

    # Cache miss - query database
    user := db_query_user(user_id)

    # Store in cache
    cache.put(key, user)
    free(key)

    user
}
```

### Session Storage

```vais
F validate_session(cache: Cache, session_id: i64) -> i64 {
    user := cache.get(session_id)

    I user == 0 {
        # Session not in cache or expired
        R 0
    }

    # Valid session
    1
}

F store_session(cache: Cache, session_id: i64, user_data: i64) -> i64 {
    cache.put(session_id, user_data)
}
```

### API Response Cache

```vais
F fetch_with_cache(cache: Cache, url: i64) -> i64 {
    # Check cache
    response := cache.get(url)
    I response != 0 {
        puts_ptr("Cache hit")
        R response
    }

    # Cache miss - make HTTP request
    puts_ptr("Cache miss - fetching...")
    response = http_get(url)

    # Store in cache
    cache.put(url, response)

    response
}
```

## Constants

- `DEFAULT_CAPACITY = 100` - Default cache capacity
- `MAX_KEY_LEN = 256` - Maximum key length
- `MAX_VALUE_LEN = 1024` - Maximum value length

## Implementation Notes

- This is a simplified O(n) implementation using linear search
- Keys and values are copied (not borrowed)
- All strings must be freed by the cache
- Production LRU caches use hash tables + doubly-linked lists for O(1) operations

## Memory Management

- Keys and values are allocated and copied on `put()`
- Memory is freed on eviction, `remove()`, `clear()`, or `free()`
- Always call `cache.free()` when done to prevent leaks

## Limitations

- O(n) operations (not O(1) like hash-based LRU)
- Fixed capacity (no dynamic resizing)
- String keys and values only
- No TTL (time-to-live) support
- No cache statistics

## License

MIT
