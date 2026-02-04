# Vais Data Pipeline

A high-performance CSV data processing pipeline built with Vais, featuring streaming reads, transformations, filtering, and JSON output generation.

## Features

### CSV Processing
- **Streaming Reader**: Memory-efficient line-by-line CSV file reading
- **Field Parsing**: Parse CSV fields with comma separation and quote handling
- **Header Support**: Skip or process header rows

### Data Transformation
- **Filtering**: Filter rows based on field values and thresholds
- **Mapping**: Transform field values (e.g., uppercase conversion)
- **Aggregation**: Calculate statistics (sum, average, min, max, count)

### Output Generation
- **JSON Writer**: Generate formatted JSON arrays and objects
- **Metadata**: Include processing statistics in output
- **Performance Metrics**: Report execution time and row counts

## Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│ CSV Reader  │───▶│ Transformer  │───▶│ JSON Writer │
└─────────────┘    └──────────────┘    └─────────────┘
      │                   │                    │
   Streams            Filters              Outputs
   CSV data           Maps                 JSON
                      Aggregates
```

## Project Structure

```
vais-datapipe/
├── vais.toml              # Package manifest
├── README.md              # This file
└── src/
    ├── main.vais          # Entry point and CLI handling
    ├── csv_reader.vais    # CSV file streaming and parsing
    ├── transformer.vais   # Data transformation operations
    ├── json_writer.vais   # JSON output generation
    └── pipeline.vais      # Pipeline orchestration
```

## Usage

### Basic Usage

```bash
# Process CSV file to JSON
vais-datapipe input.csv output.json

# Show help
vais-datapipe --help
```

### Example Input (data.csv)

```csv
name,age,city,salary
Alice,35,NYC,75000
Bob,28,SF,65000
Charlie,42,LA,85000
Diana,31,NYC,70000
```

### Example Output (output.json)

```json
[
  {
    "line": 1,
    "fields": 4
  },
  {
    "line": 2,
    "fields": 4
  }
]
```

## Pipeline Configuration

The pipeline can be configured with:

- **Input Path**: CSV file to process
- **Output Path**: JSON file to generate
- **Filter Settings**: Field index and threshold for row filtering
- **Aggregate Field**: Field to calculate statistics on
- **Skip Header**: Whether to skip the first row

## Implementation Details

### CSV Reader (`csv_reader.vais`)
- Opens file handle with `fopen`
- Reads lines with `fgets` into buffer
- Parses fields into structured rows
- Manages memory allocation for buffers

### Transformer (`transformer.vais`)
- Filters rows based on field comparisons
- Maps fields with transformations
- Accumulates aggregate statistics
- Updates running calculations

### JSON Writer (`json_writer.vais`)
- Writes formatted JSON arrays and objects
- Handles proper comma placement
- Supports string and number fields
- Writes metadata and statistics

### Pipeline (`pipeline.vais`)
- Orchestrates reader → transformer → writer flow
- Tracks processing metrics
- Handles cleanup and error cases
- Reports execution results

## Building

```bash
# From project root
./build.sh

# Or manually
vaisc src/csv_reader.vais src/transformer.vais src/json_writer.vais src/pipeline.vais src/main.vais -o vais-datapipe
```

## Performance

The pipeline is designed for efficiency:

- **Streaming**: Processes one line at a time (O(1) memory)
- **No Allocations**: Reuses buffers where possible
- **Single Pass**: Completes in one file traversal
- **Timing Metrics**: Reports processing time in milliseconds

## Example Processing Statistics

```
=== Pipeline Execution Result ===
Status: SUCCESS
Processed rows: 4
Filtered rows: 0
Elapsed time (ms): 45
=== Done ===
```

## Extern Functions Used

Standard C library functions:
- **File I/O**: `fopen`, `fclose`, `fgets`, `fputs`, `fprintf`
- **Memory**: `malloc`, `free`, `memset`
- **Strings**: `strlen`, `strchr`, `strncpy`, `strcmp`, `sprintf`
- **Conversion**: `atoi`, `atof`, `toupper`
- **Timing**: `time_get_ms`
- **Output**: `printf`, `puts`

## Future Enhancements

- [ ] Support multiple delimiter types (tabs, semicolons)
- [ ] Advanced quote and escape handling
- [ ] More transformation functions (lowercase, trim, replace)
- [ ] Multiple aggregation operations in one pass
- [ ] CSV output format option
- [ ] Parallel processing for large files
- [ ] Custom filter expressions
- [ ] Field type inference

## License

MIT
