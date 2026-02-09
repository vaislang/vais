# vais-csv

CSV parser and generator for Vais - parse and write CSV files with proper escaping.

## Features

- **CSV Parser**: Parse CSV lines into structured row data
- **CSV Generator**: Generate properly formatted CSV output
- **RFC 4180 Compliant**: Handles quoted fields, embedded commas, and quote escaping
- **Header Support**: Parse and work with CSV headers
- **Memory Safe**: Manual memory management with clear ownership
- **Lightweight**: Pure Vais implementation with no external dependencies

## Installation

Add to your `vais.toml`:

```toml
[dependencies]
vais-csv = "1.0.0"
```

## Usage

### Parsing CSV

```vais
# Parse a simple CSV line
row := csv_parse_line("name,age,city")

count := csv_row_count(row)  # 3

# Access fields by index
name := csv_row_get(row, 0)   # "name"
age := csv_row_get(row, 1)    # "age"
city := csv_row_get(row, 2)   # "city"

# Clean up when done
csv_row_free(row)
```

### Handling Quoted Fields

```vais
# Parse CSV with quoted field containing comma
row := csv_parse_line("Alice,30,\"New York, NY\"")

name := csv_row_get(row, 0)   # "Alice"
age := csv_row_get(row, 1)    # "30"
city := csv_row_get(row, 2)   # "New York, NY"

csv_row_free(row)
```

### Generating CSV

```vais
# Create array of field strings
fields := malloc(3 * 8)
store_i64(fields, str_to_ptr("Alice"))
store_i64(fields + 8, str_to_ptr("30"))
store_i64(fields + 16, str_to_ptr("New York, NY"))

# Generate CSV line
csv_line := csv_write_row(fields, 3)
puts(csv_line)  # Alice,30,"New York, NY"

free(csv_line)
free(fields)
```

### Working with Headers

```vais
# Parse header row
header := csv_parse_header("name,email,age")

# Find column index by name
email_idx := csv_header_find(header, "email")  # 1

# Parse data row
data_row := csv_parse_line("Alice,alice@example.com,30")

# Access field by name using header
I email_idx >= 0 {
    email := csv_row_get(data_row, email_idx)
    puts(email)  # "alice@example.com"
}

csv_row_free(header)
csv_row_free(data_row)
```

### Creating Rows

```vais
# Create empty row
row := csv_row_new()

# Add fields
csv_row_add(row, str_copy("Alice"))
csv_row_add(row, str_copy("30"))
csv_row_add(row, str_copy("New York"))

# Convert to CSV string
csv_line := csv_row_to_string(row)
puts(csv_line)

free(csv_line)
csv_row_free(row)
```

### Handling Special Characters

```vais
# Quotes are automatically escaped when needed
row := csv_parse_line("\"He said \"\"hello\"\"\",world")

field1 := csv_row_get(row, 0)  # He said "hello"
field2 := csv_row_get(row, 1)  # world

csv_row_free(row)
```

## API Reference

### Row Operations

- `csv_row_new() -> i64` - Create empty CSV row
- `csv_row_get(row: i64, idx: i64) -> i64` - Get field at index
- `csv_row_count(row: i64) -> i64` - Get number of fields
- `csv_row_add(row: i64, field: i64) -> i64` - Add field to row
- `csv_row_free(row: i64) -> i64` - Free row and all fields

### Parsing

- `csv_parse_line(line: i64) -> i64` - Parse CSV line into row
- `csv_parse_header(line: i64) -> i64` - Parse header row

### Generation

- `csv_row_to_string(row: i64) -> i64` - Convert row to CSV string
- `csv_write_row(fields: i64, count: i64) -> i64` - Generate CSV from field array

### Header Utilities

- `csv_header_find(header: i64, name: i64) -> i64` - Find column index by name (returns -1 if not found)

### Utilities

- `csv_field_copy(field: i64) -> i64` - Copy field string
- `csv_row_from_strings(strings: i64, count: i64) -> i64` - Create row from string array

## CSV Format Rules

The library implements RFC 4180 CSV format:

1. **Fields**: Separated by commas
2. **Quotes**: Fields containing commas, quotes, or newlines must be quoted
3. **Escaping**: Quotes within quoted fields are doubled (`""`)
4. **Empty fields**: Represented as empty strings between commas

### Examples

```csv
# Simple fields
Alice,Bob,Charlie

# Quoted field with comma
Alice,"New York, NY",30

# Quoted field with quote
"He said ""hello""",world

# Empty fields
Alice,,30

# Mixed
"Smith, John",42,"San Francisco, CA"
```

## Limitations

- Maximum fields per row: 128
- Maximum field length: 2048 bytes
- Buffer size: 8192 bytes for CSV line generation
- No multi-line field support (newlines in fields not handled)

## Testing

Run the test suite:

```bash
cargo run --bin vaisc -- tests/test_csv.vais
./test_csv
```

## Examples

### Processing CSV File

```vais
# Parse multiple rows
lines := ["name,age,city", "Alice,30,NYC", "Bob,25,LA"]

header := csv_parse_line(lines[0])
name_idx := csv_header_find(header, "name")
age_idx := csv_header_find(header, "age")

i := mut 1
L i < 3 {
    row := csv_parse_line(lines[i])

    name := csv_row_get(row, name_idx)
    age := csv_row_get(row, age_idx)

    puts(name)
    puts(" is ")
    puts(age)
    puts(" years old")
    putchar(10)

    csv_row_free(row)
    i = i + 1
}

csv_row_free(header)
```

## License

MIT
