# Vais DAP (Debug Adapter Protocol) Server

A fully-featured Debug Adapter Protocol implementation for the Vais programming language.

## Architecture

```
┌─────────────────────────────────────────┐
│       DAP Server (this crate)           │
├─────────────────────────────────────────┤
│ • Protocol handling (JSON-RPC)          │
│ • Session management                    │
│ • Breakpoint management                 │
│ • Stack trace / Variables               │
├─────────────────────────────────────────┤
│   Debugger Backend (LLDB)               │
├─────────────────────────────────────────┤
│ • Process control                       │
│ • Event handling                        │
│ • Memory inspection                     │
└─────────────────────────────────────────┘
```

## Features

- **Full DAP Protocol Support**:
  - Initialize/ConfigurationDone lifecycle
  - Launch and attach debugging
  - Source and function breakpoints
  - Conditional and hit-count breakpoints
  - Step over, step in, step out
  - Variable inspection (locals, arguments, registers)
  - Call stack navigation
  - Expression evaluation
  - Memory read/write operations
  - Disassembly view

- **LLDB Backend**:
  - Cross-platform process control
  - DWARF debug info parsing
  - Register inspection
  - Advanced breakpoint conditions

- **VSCode Integration**:
  - Auto-generated launch.json configurations
  - Inline variable values during debugging
  - Syntax-aware breakpoint validation

## Building

```bash
# Build the DAP server
cargo build --release -p vais-dap

# The binary will be at:
target/release/vais-dap
```

## Running

### Standalone Mode (stdio)

```bash
vais-dap
```

The server communicates via stdin/stdout using the DAP protocol.

### TCP Mode (for testing)

```bash
vais-dap --port 4711
```

## Testing

The crate includes comprehensive E2E integration tests that verify the full DAP protocol lifecycle.

### Running All Tests

```bash
cargo test -p vais-dap
```

### Running Specific Test Suites

```bash
# Protocol message round-trip tests
cargo test -p vais-dap test_initialize_round_trip
cargo test -p vais-dap test_configuration_done_round_trip
cargo test -p vais-dap test_disconnect_round_trip

# Breakpoint tests
cargo test -p vais-dap test_set_breakpoints
cargo test -p vais-dap test_set_function_breakpoints
cargo test -p vais-dap test_clear_breakpoints

# Stack trace tests
cargo test -p vais-dap test_stack_trace_request
cargo test -p vais-dap test_threads_request

# Variable inspection tests
cargo test -p vais-dap test_scopes_request
cargo test -p vais-dap test_variables_request
cargo test -p vais-dap test_set_variable_request

# Execution control tests
cargo test -p vais-dap test_continue_request
cargo test -p vais-dap test_step_over_request
cargo test -p vais-dap test_step_in_request
cargo test -p vais-dap test_step_out_request
cargo test -p vais-dap test_pause_request

# Evaluate tests
cargo test -p vais-dap test_evaluate_request

# Memory operations
cargo test -p vais-dap test_read_memory_request
cargo test -p vais-dap test_disassemble_request

# Error handling tests
cargo test -p vais-dap test_invalid_request
cargo test -p vais-dap test_missing_arguments
cargo test -p vais-dap test_operation_without_session

# Full session lifecycle
cargo test -p vais-dap test_full_session_lifecycle
```

### Test Coverage

The test suite covers:

1. **Protocol Message Round-Trip Tests** (3 tests)
   - Initialize request/response
   - ConfigurationDone request/response
   - Disconnect request/response

2. **Breakpoint Tests** (3 tests)
   - Setting source breakpoints
   - Setting function breakpoints
   - Clearing breakpoints

3. **Stack Trace Tests** (2 tests)
   - Stack trace requests
   - Thread enumeration

4. **Variable Inspection Tests** (3 tests)
   - Scopes (locals, arguments, registers)
   - Variable retrieval
   - Variable modification

5. **Execution Control Tests** (5 tests)
   - Continue execution
   - Step over (next)
   - Step in
   - Step out
   - Pause

6. **Evaluate Tests** (1 test)
   - Expression evaluation

7. **Memory Operations Tests** (2 tests)
   - Memory read
   - Disassembly

8. **Error Handling Tests** (3 tests)
   - Invalid commands
   - Missing arguments
   - Operations without active session

9. **Full Session Lifecycle Test** (1 test)
   - Complete debugging workflow from initialization to disconnect

**Total: 23 E2E integration tests**

## VSCode Extension Setup

The Vais VSCode extension includes built-in DAP support.

### Installation

1. Install the VSCode extension:
   ```bash
   cd vscode-vais
   npm install
   npm run compile
   ```

2. Install the DAP server:
   ```bash
   cargo install --path crates/vais-dap
   ```

3. Configure the path in VSCode settings (optional):
   ```json
   {
     "vais.debugAdapter.path": "vais-dap"
   }
   ```

### Generating launch.json

Use the command palette:
1. Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
2. Run: "Vais: Generate Debug Configuration"

Or use the pre-configured template at `vscode-vais/launch-template.json`.

### Example launch.json

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "vais",
      "request": "launch",
      "name": "Debug Vais Program",
      "program": "${workspaceFolder}/main.vais",
      "stopOnEntry": true,
      "autoCompile": true,
      "optLevel": 0,
      "args": [],
      "cwd": "${workspaceFolder}"
    },
    {
      "type": "vais",
      "request": "launch",
      "name": "Debug Current File",
      "program": "${file}",
      "stopOnEntry": true,
      "autoCompile": true
    },
    {
      "type": "vais",
      "request": "attach",
      "name": "Attach to Process",
      "pid": 0,
      "stopOnAttach": true
    }
  ]
}
```

### Launch Configuration Options

#### Launch Request

- `program` (required): Path to the Vais source file to debug
- `binary` (optional): Path to pre-compiled binary
- `args` (optional): Command-line arguments for the program
- `cwd` (optional): Working directory
- `stopOnEntry` (optional, default: true): Stop at entry point
- `autoCompile` (optional, default: true): Automatically compile before debugging
- `optLevel` (optional, default: 0): Optimization level (0-3)
- `env` (optional): Environment variables

#### Attach Request

- `pid` (required): Process ID to attach to
- `program` (optional): Path to program binary (for symbol loading)
- `stopOnAttach` (optional, default: true): Stop all threads when attaching

## Usage in Other Editors

### Neovim (nvim-dap)

```lua
local dap = require('dap')

dap.adapters.vais = {
  type = 'executable',
  command = 'vais-dap',
}

dap.configurations.vais = {
  {
    type = 'vais',
    request = 'launch',
    name = 'Launch Vais Program',
    program = '${file}',
    stopOnEntry = true,
    autoCompile = true,
  },
}
```

### Emacs (dap-mode)

```elisp
(require 'dap-mode)

(dap-register-debug-provider
 "vais"
 (lambda (conf)
   (plist-put conf :dap-server-path '("vais-dap"))
   conf))

(dap-register-debug-template
 "Vais :: Launch"
 (list :type "vais"
       :request "launch"
       :name "Launch Vais Program"
       :program "${file}"
       :stopOnEntry t
       :autoCompile t))
```

## Protocol Details

The DAP server implements the Debug Adapter Protocol as specified at:
https://microsoft.github.io/debug-adapter-protocol/

### Message Format

All messages follow the DAP base protocol:

```
Content-Length: <length>\r\n
\r\n
<JSON message>
```

### Request/Response Flow

```
Client → Server: Request
Server → Client: Response (with same request_seq)
Server → Client: Event (async notifications)
```

### Error Handling

Errors are returned with:
- `success: false`
- `message`: Human-readable error description
- `body.error`: Structured error details (optional)

Error codes:
- 1000-1999: Protocol/serialization errors
- 2000-2999: Debugger backend errors
- 3000-3999: DWARF/debug info errors
- 4000-4999: Unsupported operations
- 5000-5999: Timeout errors

## Dependencies

- **tokio**: Async runtime for I/O operations
- **serde/serde_json**: Protocol serialization
- **LLDB**: Native debugger backend (system dependency)
- **gimli/object**: DWARF debug info parsing
- **vais-lexer/parser/ast**: Source mapping

## Known Limitations

1. **LLDB Required**: The DAP server requires LLDB to be installed on the system
2. **Debug Info**: Requires programs to be compiled with debug info (`-g` flag)
3. **Single Process**: Currently supports debugging one process at a time
4. **Platform Support**: Tested on Linux and macOS; Windows support via LLDB

## Future Enhancements

- [ ] GDB backend support
- [ ] Remote debugging over TCP
- [ ] Multi-process debugging
- [ ] Time-travel debugging (rr integration)
- [ ] Advanced expression evaluation (Vais REPL integration)
- [ ] Hot reload support during debugging
- [ ] GPU debugging support
- [ ] WebAssembly debugging

## Contributing

When adding new DAP features:

1. Implement the handler in `src/server.rs`
2. Add protocol types in `src/protocol/`
3. Add E2E tests in `tests/e2e_tests.rs`
4. Update this README

Test your changes with:
```bash
cargo test -p vais-dap
cargo clippy -p vais-dap
```

## License

See the main project LICENSE file.
