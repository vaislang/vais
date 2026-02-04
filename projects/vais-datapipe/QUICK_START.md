# Vais Data Pipeline - Quick Start

## What It Does

Processes CSV files with filtering, transformation, and aggregation, then outputs to JSON.

## Project Overview

```
CSV File → Read → Filter → Transform → Aggregate → JSON Output
```

## File Structure

- `src/csv_reader.vais` - Stream CSV files line by line
- `src/transformer.vais` - Filter and transform data
- `src/json_writer.vais` - Generate JSON output
- `src/pipeline.vais` - Orchestrate the pipeline
- `src/main.vais` - CLI entry point

## Key Features

✅ **Streaming**: O(1) memory, processes large files
✅ **Filtering**: Keep rows based on field criteria
✅ **Aggregation**: Calculate sum, average, min, max
✅ **JSON Output**: Formatted array with metadata
✅ **Performance**: Track processing time and row counts

## Example Usage

```bash
# Build the project
./build.sh

# Run the pipeline
vais-datapipe data.csv output.json
```

## Input Example (data.csv)

```csv
name,age,city,salary
Alice,35,NYC,75000
Bob,28,SF,65000
Charlie,42,LA,85000
```

## Output Example (output.json)

```json
[
  {"line": 1, "fields": 4},
  {"line": 2, "fields": 4},
  {"line": 3, "fields": 4}
]
```

## Architecture

### CsvReader
- Opens file with `fopen`
- Reads lines with `fgets`
- Parses fields into rows
- Manages buffers

### Transformer
- Filters rows by threshold
- Maps field transformations
- Tracks running statistics
- Configurable operations

### JsonWriter
- Writes JSON arrays
- Formats objects
- Handles strings and numbers
- Outputs metadata

### Pipeline
- Chains components
- Manages state
- Reports results
- Handles errors

## Configuration

Edit in `main.vais`:

```vais
C configured := PipelineConfig {
    input_path: config.input_path,
    output_path: config.output_path,
    filter_enabled: 1,      # Enable filtering
    filter_field: 2,        # Filter on field 2
    filter_threshold: 30,   # Keep if > 30
    aggregate_field: 3,     # Aggregate field 3
    skip_header: 1,         # Skip first row
}
```

## Code Statistics

- **Total**: 870+ lines of Vais code
- **Structs**: 7 data structures
- **Functions**: 50+ functions
- **Extern**: 20+ C library functions

## Performance

- Streaming: O(1) memory
- Single-pass: O(n) time
- Buffer reuse: Minimal allocations

## Learning Resources

See `README.md` for:
- Detailed documentation
- Architecture diagrams
- API reference
- Future enhancements
