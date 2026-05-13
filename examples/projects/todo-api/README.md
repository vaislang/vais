# Todo API - REST API with SQLite CRUD

A complete REST API server implementation in Vais demonstrating HTTP routing, JSON handling, and SQLite database operations.

## Features

- REST API with full CRUD operations
- SQLite database persistence
- JSON request/response handling
- HTTP routing with path parameters
- TCP socket server implementation

## Project Structure

```
todo-api/
├── main.vais      # HTTP server and routing logic
├── models.vais    # Todo struct and JSON serialization
├── db.vais        # SQLite CRUD operations
└── README.md      # This file
```

## API Endpoints

### GET /todos
Get all todos.

**Response:** 200 OK
```json
[
  {"id":1,"title":"Buy milk","completed":false},
  {"id":2,"title":"Write code","completed":true}
]
```

### POST /todos
Create a new todo.

**Request:**
```json
{"title":"New task"}
```

**Response:** 201 Created
```json
{"id":3,"title":"New task","completed":false}
```

### GET /todos/:id
Get a single todo by ID.

**Response:** 200 OK
```json
{"id":1,"title":"Buy milk","completed":false}
```

**Response:** 404 Not Found
```json
{"error":"Not found"}
```

### DELETE /todos/:id
Delete a todo by ID.

**Response:** 204 No Content

**Response:** 404 Not Found
```json
{"error":"Not found"}
```

## Build & Run

### Prerequisites

- Vais compiler (`vaisc`)
- SQLite3 library
- LLVM/clang toolchain

### Compile

```bash
cd examples/projects/todo-api
vaisc main.vais -o todo-api
```

### Run

```bash
./todo-api
```

Server will start on port 8080:
```
Starting Todo API server...
Database initialized
Server listening on port 8080
```

## Usage Examples

### Create a todo

```bash
curl -X POST http://localhost:8080/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Learn Vais"}'
```

### Get all todos

```bash
curl http://localhost:8080/todos
```

### Get a specific todo

```bash
curl http://localhost:8080/todos/1
```

### Delete a todo

```bash
curl -X DELETE http://localhost:8080/todos/1
```

## Implementation Details

### HTTP Server

The server uses low-level TCP socket operations:
- `__tcp_listen(port)` - Create listening socket
- `__tcp_accept(fd)` - Accept client connections
- `__tcp_recv(fd, buf, len)` - Read HTTP request
- `__tcp_send(fd, data, len)` - Send HTTP response
- `__tcp_close(fd)` - Close connection

### Request Parsing

- HTTP method extraction from request line
- Path parsing with query string handling
- Route parameter extraction (e.g., `/todos/:id`)
- JSON body parsing for POST requests

### SQLite Integration

The `db.vais` module uses SQLite3 C API:
- `sqlite3_open()` - Open database connection
- `sqlite3_prepare_v2()` - Compile SQL statement
- `sqlite3_bind_text/int64()` - Bind parameters
- `sqlite3_step()` - Execute statement
- `sqlite3_column_*()` - Read result columns
- `sqlite3_finalize()` - Free statement

### JSON Serialization

Manual JSON construction in `models.vais`:
- `todo_to_json(todo)` - Single todo to JSON object
- `todos_array_to_json(todos, count)` - Array to JSON
- String concatenation for building JSON strings

## Database Schema

```sql
CREATE TABLE todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    completed INTEGER DEFAULT 0
);
```

The database file `todos.db` is created automatically in the current directory.

## Error Handling

- 200 OK - Successful GET
- 201 Created - Successful POST
- 204 No Content - Successful DELETE
- 400 Bad Request - Invalid request body
- 404 Not Found - Resource not found
- 405 Method Not Allowed - Unsupported HTTP method
- 500 Internal Server Error - Database or server error

## Vais Language Features Demonstrated

- Struct definitions (`S Todo`)
- Function definitions (`F handle_request`)
- Module imports (`U models`, `U db`)
- Extern function declarations (`X F sqlite3_open`)
- Control flow (`I/E/L/B/R`)
- Memory management (`malloc`, `store_i64`, `load_i64`)
- String operations (`strlen`, `str_to_ptr`, `ptr_to_str`)
- TCP socket operations
- SQLite database operations

## Limitations

- No concurrent connection handling (single-threaded)
- No request body size limits
- No authentication/authorization
- No query parameters support
- Simplified JSON parsing (supports only `{"title":"..."}` format)
- Maximum 100 todos in GET /todos response

## Future Enhancements

- Add PUT endpoint for updating todos
- Add PATCH endpoint for toggling completed status
- Query parameters for filtering (e.g., `?completed=true`)
- Pagination support
- Request validation and error messages
- Concurrent connection handling with threading
- Environment variable configuration (port, database path)
- Comprehensive JSON parser
- Request logging
