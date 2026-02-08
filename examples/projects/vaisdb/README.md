# VaisDB — Storage Engine in Vais

A minimal database storage engine demonstrating Vais systems programming capabilities.

## Architecture

```
main.vais      — Integration tests & demo
storage.vais   — Table API (insert/get/scan/flush)
page.vais      — 4KB slotted page manager + buffer pool
row.vais       — TLV row serialization (i64/str/bool/null)
btree.vais     — In-memory B-Tree index (order 8)
```

## Features

- **Slotted pages**: Fixed 4KB pages with slot directory, data grows downward
- **Row format**: Type-Length-Value with varint encoding for strings
- **B-Tree index**: Primary key lookups in O(log n), range scans
- **Buffer pool**: Up to 256 pages, flush to disk via `fwrite`
- **Auto-increment**: Primary keys assigned automatically on insert

## Usage

```bash
vaisc examples/projects/vaisdb/main.vais -o vaisdb && ./vaisdb
```

## Page Layout

```
+------------------+
| Header (64B)     |  page_id, num_slots, free_offset, data_end, next_page, type
+------------------+
| Slot 0 (8B)      |  offset → Row 0
| Slot 1 (8B)      |  offset → Row 1
| ...              |
+------------------+
| Free Space       |
+------------------+
| Row 1 data       |  ← grows upward
| Row 0 data       |
+------------------+
```

## Row Format

```
[col_count: 1B] [col1_type: 1B] [col1_data] [col2_type: 1B] [col2_data] ...

Types: 0=NULL, 1=I64(8B LE), 2=STR(varint_len + bytes), 3=BOOL(1B)
```
