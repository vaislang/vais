# BTreeMap API Reference

> Self-balancing ordered map using B-tree (stores i64 key-value pairs in sorted order)

## Import

```vais
U std/btreemap
U std/option
```

## Overview

The `btreemap` module provides a B-tree based ordered map implementation with:
- Sorted key storage (in-order traversal)
- Self-balancing structure (min degree t=2)
- Efficient search, insert, and iteration
- Free function API (no methods)
- Optional error handling via `Option<T>`

**Note:** This implementation uses a free function API, not struct methods. The map is represented as an opaque pointer (i64).

## Constants

- **MIN_DEGREE**: 2 (minimum degree t, so each node has at most 2*t-1 = 3 keys)
- **MAX_KEYS**: 3
- **MAX_CHILDREN**: 4

## Data Structure

### Internal Node Layout
Each B-tree node is 96 bytes:
- `[0]` = num_keys (number of keys in this node)
- `[8]` = is_leaf (1 = leaf, 0 = internal)
- `[16]` = key[0]
- `[24]` = value[0]
- `[32]` = key[1]
- `[40]` = value[1]
- `[48]` = key[2]
- `[56]` = value[2]
- `[64]` = child[0] (pointer)
- `[72]` = child[1]
- `[80]` = child[2]
- `[88]` = child[3]

### Map Layout
- `[0]` = root node pointer
- `[8]` = size (number of entries)

## Core Functions

### Creation and Access

| Function | Signature | Description |
|----------|-----------|-------------|
| `btreemap_new` | `F btreemap_new() -> i64` | Create empty B-tree map |
| `btreemap_root` | `F btreemap_root(map: i64) -> i64` | Get root node pointer |
| `btreemap_size` | `F btreemap_size(map: i64) -> i64` | Get number of entries |
| `btreemap_free` | `F btreemap_free(map: i64) -> i64` | Free all memory |

### Get Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `btreemap_get` | `F btreemap_get(map: i64, key: i64) -> i64` | Get value by key (returns 0 if not found) |
| `btreemap_get_opt` | `F btreemap_get_opt(map: i64, key: i64) -> Option<i64>` | Get value as Option (Some/None) |
| `btreemap_contains` | `F btreemap_contains(map: i64, key: i64) -> i64` | Check if key exists (1=yes, 0=no) |

### Insert Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `btreemap_put` | `F btreemap_put(map: i64, key: i64, value: i64) -> i64` | Insert or update key-value pair (returns 1) |

### Min/Max

| Function | Signature | Description |
|----------|-----------|-------------|
| `btreemap_min_key` | `F btreemap_min_key(map: i64) -> i64` | Get minimum key (0 if empty) |
| `btreemap_max_key` | `F btreemap_max_key(map: i64) -> i64` | Get maximum key (0 if empty) |

### Iteration

| Function | Signature | Description |
|----------|-----------|-------------|
| `btreemap_foreach` | `F btreemap_foreach(map: i64, callback: i64, context: i64) -> i64` | Traverse map in-order with callback |

## Internal Helper Functions

### Node Management

| Function | Description |
|----------|-------------|
| `btree_node_new(is_leaf: i64) -> i64` | Create new node |
| `btree_node_num_keys(node: i64) -> i64` | Get number of keys |
| `btree_node_is_leaf(node: i64) -> i64` | Check if node is leaf |
| `btree_node_get_key(node: i64, i: i64) -> i64` | Get key at index |
| `btree_node_get_value(node: i64, i: i64) -> i64` | Get value at index |
| `btree_node_set_key(node: i64, i: i64, key: i64) -> i64` | Set key at index |
| `btree_node_set_value(node: i64, i: i64, value: i64) -> i64` | Set value at index |
| `btree_node_get_child(node: i64, i: i64) -> i64` | Get child pointer |
| `btree_node_set_child(node: i64, i: i64, child: i64) -> i64` | Set child pointer |
| `btree_node_set_num_keys(node: i64, n: i64) -> i64` | Set number of keys |

### Search

| Function | Description |
|----------|-------------|
| `btree_search(node: i64, key: i64) -> i64` | Search for key in tree |
| `btree_search_rec(node: i64, key: i64, i: i64) -> i64` | Recursive search helper |

### Insertion

| Function | Description |
|----------|-------------|
| `btree_insert_nonfull(node: i64, key: i64, value: i64) -> i64` | Insert into non-full node |
| `btree_split_child(parent: i64, i: i64, child: i64) -> i64` | Split full child node |
| `btree_shift_keys_right(node: i64, n: i64, from: i64) -> i64` | Shift keys right |
| `btree_shift_children_right(node: i64, n: i64, from: i64) -> i64` | Shift children right |
| `btree_find_child_index(node: i64, key: i64, i: i64) -> i64` | Find child index for key |
| `btree_insert_in_leaf(node: i64, key: i64, value: i64, n: i64) -> i64` | Insert into leaf |
| `btree_find_insert_pos(node: i64, key: i64, i: i64) -> i64` | Find insertion position |
| `btree_update_if_exists(node: i64, key: i64, value: i64) -> i64` | Update existing key |
| `btree_update_rec(node: i64, key: i64, value: i64, i: i64) -> i64` | Recursive update |

### Traversal

| Function | Description |
|----------|-------------|
| `btree_traverse(node: i64, callback: i64, context: i64) -> i64` | In-order traversal |
| `btree_traverse_rec(node: i64, callback: i64, context: i64, i: i64) -> i64` | Recursive traversal |
| `btree_find_min(node: i64) -> i64` | Find minimum key in subtree |
| `btree_find_max(node: i64) -> i64` | Find maximum key in subtree |

### Memory Management

| Function | Description |
|----------|-------------|
| `btree_free_node(node: i64) -> i64` | Free node and children |
| `btree_free_children(node: i64, i: i64) -> i64` | Recursively free children |
| `btreemap_set_root(map: i64, root: i64) -> i64` | Set root pointer |
| `btreemap_inc_size(map: i64) -> i64` | Increment size counter |

## Examples

### Basic Usage

```vais
U std/btreemap

F main() -> i64 {
    # Create new map
    map := btreemap_new()

    # Insert key-value pairs
    btreemap_put(map, 5, 50)
    btreemap_put(map, 2, 20)
    btreemap_put(map, 8, 80)
    btreemap_put(map, 1, 10)

    # Get values
    val := btreemap_get(map, 5)  # Returns 50

    # Check existence
    exists := btreemap_contains(map, 2)  # Returns 1
    not_exists := btreemap_contains(map, 99)  # Returns 0

    # Get size
    size := btreemap_size(map)  # Returns 4

    # Free memory
    btreemap_free(map)
    0
}
```

### Using Option for Error Handling

```vais
U std/btreemap
U std/option

F main() -> i64 {
    map := btreemap_new()

    btreemap_put(map, 42, 100)

    # Use Option-based get
    opt := btreemap_get_opt(map, 42)

    M opt {
        Some(v) => {
            # Key found, v is the value
            v  # 100
        },
        None => {
            # Key not found
            0
        }
    }
}
```

### Min/Max Keys

```vais
U std/btreemap

F main() -> i64 {
    map := btreemap_new()

    btreemap_put(map, 10, 100)
    btreemap_put(map, 5, 50)
    btreemap_put(map, 20, 200)
    btreemap_put(map, 15, 150)

    # Keys are stored in sorted order
    min := btreemap_min_key(map)  # Returns 5
    max := btreemap_max_key(map)  # Returns 20

    btreemap_free(map)
    0
}
```

### Update Existing Keys

```vais
U std/btreemap

F main() -> i64 {
    map := btreemap_new()

    # Insert
    btreemap_put(map, 1, 10)
    val1 := btreemap_get(map, 1)  # Returns 10

    # Update (same key, new value)
    btreemap_put(map, 1, 20)
    val2 := btreemap_get(map, 1)  # Returns 20

    # Size doesn't change on update
    size := btreemap_size(map)  # Still 1

    btreemap_free(map)
    0
}
```

### Iteration with Callback

```vais
U std/btreemap

# Callback function receives (key, value, context)
# Returns 0 to continue, 1 to stop
F print_entry(key: i64, value: i64, context: i64) -> i64 {
    # Print or process key-value pair
    # Note: actual implementation would need function pointer support
    0  # Continue
}

F main() -> i64 {
    map := btreemap_new()

    btreemap_put(map, 3, 30)
    btreemap_put(map, 1, 10)
    btreemap_put(map, 2, 20)

    # Traverse in sorted order: (1,10), (2,20), (3,30)
    btreemap_foreach(map, print_entry, 0)

    btreemap_free(map)
    0
}
```

### Building a Sorted Map

```vais
U std/btreemap

F main() -> i64 {
    map := btreemap_new()

    # Insert in random order
    btreemap_put(map, 50, 500)
    btreemap_put(map, 10, 100)
    btreemap_put(map, 30, 300)
    btreemap_put(map, 20, 200)
    btreemap_put(map, 40, 400)

    # B-tree automatically maintains sorted order
    # In-order traversal will visit: 10, 20, 30, 40, 50

    # Access minimum and maximum
    first_key := btreemap_min_key(map)   # 10
    last_key := btreemap_max_key(map)    # 50

    btreemap_free(map)
    0
}
```

### Checking for Keys Before Access

```vais
U std/btreemap

F safe_get(map: i64, key: i64) -> i64 {
    I btreemap_contains(map, key) == 1 {
        btreemap_get(map, key)
    } E {
        # Return default value
        -1
    }
}

F main() -> i64 {
    map := btreemap_new()
    btreemap_put(map, 5, 50)

    val1 := safe_get(map, 5)   # Returns 50
    val2 := safe_get(map, 99)  # Returns -1

    btreemap_free(map)
    0
}
```

### Memory Management

```vais
U std/btreemap

F main() -> i64 {
    # Create map
    map := btreemap_new()

    # Insert many entries
    i := 0
    L i < 100 {
        btreemap_put(map, i, i * 10)
        i = i + 1
    }

    # Verify size
    size := btreemap_size(map)  # Should be 100

    # IMPORTANT: Always free when done to prevent memory leaks
    btreemap_free(map)

    0
}
```
