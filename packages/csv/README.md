# csv

CSV parser and generator for Vais programs.

## Features

- Parse CSV lines into fields
- Handle quoted fields with embedded commas
- Generate CSV-formatted output
- Escape special characters (quotes, commas, newlines)
- Simple reader API

## Usage

```vais
U csv

F main() -> i64 {
    # Parse a CSV line
    line := "name,age,city"
    row := csv_parse_line(line)

    puts_ptr("Field count: ")
    printf("%d\n", row.field_count)

    # Access fields
    i := 0
    L i < row.field_count {
        field := row.get(i)
        I field != 0 {
            puts_ptr("Field ")
            printf("%d: ", i)
            puts_ptr(field)
        }
        i = i + 1
    }

    # Free row
    row.free()

    # Generate CSV output
    fields := malloc(3 * 8)  # Array of 3 pointers
    store_ptr(fields + 0, "John")
    store_ptr(fields + 8, "30")
    store_ptr(fields + 16, "New York")

    output := csv_write_row(fields, 3)
    puts_ptr("CSV output: ")
    puts_ptr(output)

    free(output)
    free(fields)

    0
}
```

## API

### Types

- `CsvRow` - Represents a parsed CSV row with fields
- `CsvReader` - Stateful CSV file reader

### CsvRow Methods

- `CsvRow.new() -> CsvRow` - Create new empty row
- `row.get(idx: i64) -> i64` - Get field at index
- `row.field_count` - Number of fields
- `row.add_field(field: i64) -> i64` - Add field to row
- `row.free() -> i64` - Free all memory

### CsvReader Methods

- `CsvReader.open(path: i64) -> CsvReader` - Open CSV file
- `reader.next_row() -> CsvRow` - Read next row
- `reader.close() -> i64` - Close and cleanup

### Functions

- `csv_parse_line(line: i64) -> CsvRow` - Parse CSV line
- `csv_write_row(fields: i64, count: i64) -> i64` - Generate CSV line
- `csv_needs_quoting(field: i64) -> i64` - Check if field needs quotes

## CSV Format

The parser handles:
- Comma-separated fields
- Quoted fields: `"field with, comma"`
- Escaped quotes: `"field with ""quote"""`
- Empty fields: `field1,,field3`

## Examples

```vais
# Parse complex CSV line
line := "\"Smith, John\",42,\"New \"\"York\"\"\""
row := csv_parse_line(line)

# row.field_count = 3
# row.get(0) = "Smith, John"
# row.get(1) = "42"
# row.get(2) = "New \"York\""

# Generate CSV with special characters
fields := malloc(2 * 8)
store_ptr(fields + 0, "Hello, World")
store_ptr(fields + 8, "Quote: \"test\"")
output := csv_write_row(fields, 2)
# output = "\"Hello, World\",\"Quote: \"\"test\"\"\""
```

## Notes

- All returned strings must be freed by caller
- Maximum 100 fields per row (MAX_FIELDS)
- Maximum 1024 bytes per field (MAX_FIELD_LEN)
- Row memory must be freed using `row.free()`

## License

MIT
