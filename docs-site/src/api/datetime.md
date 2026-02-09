# DateTime

Date and time manipulation.

**Module:** `std/datetime.vais`

## Types

### `DateTime`

Represents a calendar date and time.

```vais
S DateTime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
}
```

### `Duration`

Represents a time duration.

```vais
S Duration {
    seconds: i64,
    nanos: i64,
}
```

## DateTime Methods

### `from_timestamp(ts: i64) -> DateTime`

Creates a DateTime from a Unix timestamp.

```vais
dt := DateTime.from_timestamp(1707400000)
```

### `to_timestamp() -> i64`

Converts to a Unix timestamp.

```vais
ts := dt.to_timestamp()
```

### `day_of_week() -> i64`

Returns the day of the week (0 = Sunday, 6 = Saturday).

```vais
dow := dt.day_of_week()
```

### `add_days(n: i64) -> DateTime`

Returns a new DateTime with `n` days added.

```vais
tomorrow := dt.add_days(1)
```

### `is_leap_year() -> bool`

Returns `true` if the year is a leap year.

```vais
dt := DateTime { year: 2024, month: 1, day: 1, hour: 0, minute: 0, second: 0 }
dt.is_leap_year()  # true
```

## See Also

- [Time](./time.md) — low-level time functions
