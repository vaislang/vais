# validate

Input validation utilities for Vais programs.

## Features

- Email validation
- URL validation
- Numeric string validation
- Range checks
- Length constraints
- String content validation
- Pattern helpers

## Usage

```vais
U validate

F main() -> i64 {
    # Validate email
    email := "user@example.com"
    I is_email(email) {
        puts_ptr("Valid email")
    }

    # Validate URL
    url := "https://example.com"
    I is_url(url) {
        puts_ptr("Valid URL")
    }

    # Validate numeric input
    age_str := "25"
    I is_numeric(age_str) {
        age := atol_ptr(age_str)
        I in_range(age, 0, 120) {
            puts_ptr("Valid age")
        }
    }

    # Validate password length
    password := "secret123"
    I min_length(password, 8) {
        puts_ptr("Password meets minimum length")
    } E {
        puts_ptr("Password too short")
    }

    # Validate username
    username := "john_doe"
    I is_alphanumeric(username) {
        I min_length(username, 3) {
            I max_length(username, 20) {
                puts_ptr("Valid username")
            }
        }
    }

    0
}
```

## API

### Format Validation

- `is_email(s: i64) -> i64` - Check email format (user@domain.tld)
- `is_url(s: i64) -> i64` - Check URL format (http://, https://, ftp://)
- `is_numeric(s: i64) -> i64` - Check if string contains only digits
- `is_integer(s: i64) -> i64` - Check if string is valid integer
- `is_positive_integer(s: i64) -> i64` - Check if positive integer string

### Range Validation

- `in_range(value: i64, min: i64, max: i64) -> i64` - Check if value in range [min, max]
- `is_positive(value: i64) -> i64` - Check if value > 0
- `is_negative(value: i64) -> i64` - Check if value < 0
- `is_zero(value: i64) -> i64` - Check if value == 0
- `is_even(value: i64) -> i64` - Check if value is even
- `is_odd(value: i64) -> i64` - Check if value is odd

### Length Validation

- `min_length(s: i64, min: i64) -> i64` - Check minimum length
- `max_length(s: i64, max: i64) -> i64` - Check maximum length
- `exact_length(s: i64, expected: i64) -> i64` - Check exact length
- `is_empty(s: i64) -> i64` - Check if empty or whitespace only
- `is_not_empty(s: i64) -> i64` - Check if has content

### Content Validation

- `is_alpha(s: i64) -> i64` - Check if only letters (A-Z, a-z)
- `is_alphanumeric(s: i64) -> i64` - Check if only letters and digits
- `contains(haystack: i64, needle: i64) -> i64` - Check if contains substring
- `starts_with(s: i64, prefix: i64) -> i64` - Check if starts with prefix
- `ends_with(s: i64, suffix: i64) -> i64` - Check if ends with suffix

## Examples

### Form Validation

```vais
F validate_signup(username: i64, email: i64, password: i64) -> i64 {
    # Validate username
    I is_not_empty(username) == 0 {
        puts_ptr("Username required")
        R 0
    }

    I min_length(username, 3) == 0 {
        puts_ptr("Username too short")
        R 0
    }

    I is_alphanumeric(username) == 0 {
        puts_ptr("Username must be alphanumeric")
        R 0
    }

    # Validate email
    I is_email(email) == 0 {
        puts_ptr("Invalid email")
        R 0
    }

    # Validate password
    I min_length(password, 8) == 0 {
        puts_ptr("Password must be at least 8 characters")
        R 0
    }

    1  # All valid
}
```

### Number Range Validation

```vais
F validate_port(port_str: i64) -> i64 {
    I is_numeric(port_str) == 0 {
        puts_ptr("Port must be numeric")
        R 0
    }

    port := atol_ptr(port_str)

    I in_range(port, 1, 65535) == 0 {
        puts_ptr("Port must be between 1 and 65535")
        R 0
    }

    1
}
```

### File Extension Validation

```vais
F validate_image_file(filename: i64) -> i64 {
    I ends_with(filename, ".jpg") { R 1 }
    I ends_with(filename, ".jpeg") { R 1 }
    I ends_with(filename, ".png") { R 1 }
    I ends_with(filename, ".gif") { R 1 }

    puts_ptr("Invalid image file type")
    0
}
```

### Configuration Validation

```vais
F validate_config(host: i64, port: i64) -> i64 {
    # Validate host (not empty)
    I is_empty(host) {
        puts_ptr("Host cannot be empty")
        R 0
    }

    # Validate port (1-65535)
    I in_range(port, 1, 65535) == 0 {
        puts_ptr("Invalid port number")
        R 0
    }

    1
}
```

## Validation Patterns

### Required Field
```vais
I is_not_empty(field) == 0 {
    R 0  # Validation failed
}
```

### Optional Field with Constraint
```vais
I is_not_empty(field) {
    I min_length(field, 3) == 0 {
        R 0  # Invalid length
    }
}
```

### Multiple Conditions
```vais
I is_alphanumeric(username) {
    I min_length(username, 3) {
        I max_length(username, 20) {
            # All valid
        }
    }
}
```

## Notes

- All functions return 1 for valid, 0 for invalid
- Email validation is simplified (basic format check)
- URL validation only checks protocol prefix
- Empty strings are handled gracefully
- All string parameters must be null-terminated

## License

MIT
