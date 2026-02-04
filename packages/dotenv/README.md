# dotenv

.env file loader for Vais programs.

## Features

- Load environment variables from .env files
- Support for quoted and unquoted values
- Handle comments and empty lines
- Ignore `export` prefix
- Simple API

## Usage

```vais
U dotenv

F main() -> i64 {
    # Load .env file from current directory
    result := dotenv_load_default()

    I result == 0 {
        puts_ptr("Loaded .env successfully")
    } E {
        puts_ptr("Failed to load .env")
        R 1
    }

    # Access environment variables
    db_host := dotenv_get("DATABASE_HOST")
    I db_host != 0 {
        puts_ptr("Database host: ")
        puts_ptr(db_host)
    }

    # Check if variable exists
    I dotenv_has("API_KEY") {
        api_key := dotenv_get("API_KEY")
        puts_ptr("API key loaded")
    }

    0
}
```

## API

### Functions

- `dotenv_load(path: i64) -> i64` - Load .env file from path (0 = success, -1 = error)
- `dotenv_load_default() -> i64` - Load .env from current directory
- `dotenv_load_file(filename: i64) -> i64` - Load custom .env file
- `dotenv_get(key: i64) -> i64` - Get environment variable value
- `dotenv_has(key: i64) -> i64` - Check if variable exists
- `dotenv_parse_line(line: i64) -> i64` - Parse single line (internal)

### Types

- `DotEnv` - State structure (internal use)

## .env File Format

```bash
# Comments start with #
DATABASE_HOST=localhost
DATABASE_PORT=5432

# Quoted values
API_KEY="secret-key-123"
APP_NAME="My Application"

# export prefix is ignored
export NODE_ENV=production

# Empty lines are ignored

DEBUG=true
```

## Supported Features

- Key-value pairs: `KEY=value`
- Quoted values: `KEY="value with spaces"`
- Comments: `# comment`
- Empty lines
- Export prefix: `export KEY=value`

## Limitations

- No variable substitution
- No multi-line values
- No escape sequences in quoted strings
- Maximum 1024 bytes per line
- Values are set using `setenv()` with overwrite enabled

## Examples

```vais
# Load production environment
dotenv_load(".env.production")

# Check configuration
I dotenv_has("PORT") {
    port := dotenv_get("PORT")
    printf("Server port: %s\n", port)
}

# Load with error handling
result := dotenv_load(".env")
I result != 0 {
    puts_ptr("Warning: .env file not found")
    # Use default values
}
```

## File Locations

Common .env file names:
- `.env` - Default development configuration
- `.env.local` - Local overrides (not committed)
- `.env.production` - Production configuration
- `.env.test` - Test configuration

## Security Note

- Never commit .env files with secrets to version control
- Add `.env` to `.gitignore`
- Use separate .env files for different environments
- Keep production secrets out of development environments

## License

MIT
