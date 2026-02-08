# Data Pipeline - CSV to SQLite ETL

A practical example demonstrating an ETL (Extract, Transform, Load) data pipeline in Vais. This project reads CSV data, applies transformations and filters, then loads results into a database.

## Overview

This pipeline implements a three-stage ETL process:

```
CSV File → Read & Parse → Transform & Filter → Load to SQLite
```

### Data Flow

1. **Extract**: Read and parse CSV file (`sample.csv`)
2. **Transform**:
   - Filter rows where `score >= 85`
   - Calculate aggregates (average score, total score)
3. **Load**: Write filtered data to SQLite database

## Project Structure

```
data-pipeline/
├── main.vais           # Pipeline orchestrator
├── csv_reader.vais     # CSV parsing module
├── transform.vais      # Data transformation logic
├── loader.vais         # Database loader
├── sample.csv          # Sample input data (15 rows)
└── README.md           # This file
```

## Module Breakdown

### csv_reader.vais (~80 lines)
- **read_file()**: Loads entire CSV into memory buffer
- **count_lines()**: Counts rows in buffer
- **parse_line()**: Splits comma-separated fields
- **str_to_int()**: Converts string to integer
- **read_csv()**: Main function returning array of `CsvRow` structs

### transform.vais (~60 lines)
- **filter_by_score()**: Filters rows by score threshold
- **calculate_stats()**: Computes average and total scores
- **transform_data()**: Main transformation pipeline
- Returns `TransformResult` struct with filtered data and statistics

### loader.vais (~60 lines)
- **init_table()**: Creates SQL schema file
- **insert_row()**: Writes SQL INSERT statement
- **finalize_db()**: Commits transaction
- **write_csv()**: Alternative CSV output writer

### main.vais (~60 lines)
- Orchestrates three-stage pipeline
- Error handling for each stage
- Progress reporting with banners
- Produces two output files (SQL + CSV)

## Building

```bash
# From vais root directory
cargo build

# Compile the pipeline
./target/debug/vaisc examples/projects/data-pipeline/main.vais -o pipeline

# Or use cargo run
cargo run --bin vaisc -- examples/projects/data-pipeline/main.vais -o pipeline
```

## Running

```bash
# Navigate to project directory
cd examples/projects/data-pipeline

# Run the pipeline
./pipeline

# Or compile and run in one step
../../../target/debug/vaisc main.vais -o pipeline && ./pipeline
```

## Expected Output

```
==================================================
  DATA PIPELINE - CSV to SQLite ETL
==================================================

Step 1: Reading CSV data
----------------------------------------
Reading CSV file...
Successfully read 15 rows

Step 2: Transforming data (filter + aggregate)
----------------------------------------
Transforming data...
Filtered: 9 rows with score >= 85
Average score: 89
Total score: 806

Step 3: Loading to database
----------------------------------------
Loading data to database...
Initializing database table...
Database transaction committed
Successfully loaded 9 rows
Writing to CSV...
Written 9 rows to filtered_output.csv

==================================================
  PIPELINE COMPLETED SUCCESSFULLY
==================================================

Output files:
  - SQL: output.sql
  - CSV: filtered_output.csv
```

## Output Files

After running, two files are generated:

1. **output.sql**: SQLite-compatible SQL script
   ```sql
   CREATE TABLE IF NOT EXISTS high_performers (
     name TEXT,
     age INTEGER,
     score INTEGER
   );

   BEGIN TRANSACTION;

   INSERT INTO high_performers VALUES ('Alice', 25, 85);
   INSERT INTO high_performers VALUES ('Bob', 30, 92);
   ...

   COMMIT;
   ```

2. **filtered_output.csv**: CSV with filtered data
   ```csv
   name,age,score
   Alice,25,85
   Bob,30,92
   Diana,28,95
   ...
   ```

## Key Vais Patterns Demonstrated

### Memory Management
- Manual memory allocation with `malloc()`
- Buffer management for file I/O
- Struct serialization to memory

### File I/O
- `fopen()`, `fgetc()`, `fclose()` for reading
- `fprintf()` for writing formatted output
- Binary data manipulation with `load_byte()`, `store_byte()`

### Data Structures
- Struct definitions (`CsvRow`, `TransformResult`)
- Array allocation and indexing
- Pointer arithmetic for struct fields

### Control Flow
- Loop constructs (`L`) with break (`B`)
- Conditional logic (`I`/`E`)
- Error propagation with early returns

### Modular Design
- Separation of concerns (read/transform/load)
- External function declarations (`X F`)
- Struct-based data passing

## Extending This Example

1. **Add more transformations**:
   - Group by age ranges
   - Calculate percentiles
   - Normalize scores

2. **Support more CSV formats**:
   - Handle quoted fields
   - Parse different delimiters
   - Skip empty lines

3. **Database features**:
   - Real SQLite FFI integration
   - Prepared statements
   - Index creation

4. **Error handling**:
   - Validation rules
   - Data type checking
   - Recovery strategies

## Notes

- This example uses simplified SQLite integration (generates SQL text files)
- Real SQLite binding would require FFI declarations for `sqlite3_*` functions
- String operations use pointer arithmetic (no standard library string type)
- Buffer sizes are fixed (65KB for file, extendable for production)

## Related Examples

- `examples/projects/web-scraper/` - HTTP + HTML parsing
- `examples/projects/benchmark/` - Performance optimization
- `examples/file_io/` - Basic file operations
