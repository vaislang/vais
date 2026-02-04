# Vais Debugging in VSCode

This document provides comprehensive guidance on using the Vais debugger in Visual Studio Code.

## Quick Start

1. **Install the Extension**:
   - The Vais VSCode extension includes built-in debugging support
   - No additional configuration required if `vais-dap` is in your PATH

2. **Generate Debug Configuration**:
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
   - Run: "Vais: Generate Debug Configuration"
   - This creates `.vscode/launch.json` with pre-configured debug setups

3. **Start Debugging**:
   - Open a `.vais` file
   - Press `F5` or click the "Run and Debug" icon
   - Select "Debug Vais Program" from the dropdown

## Debug Configurations

### Launch Configuration (Recommended)

Compiles and runs your Vais program in debug mode:

```json
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
}
```

**Options**:

- `program`: Path to the Vais source file (supports VSCode variables)
- `stopOnEntry`: If true, stops at the entry point (main function)
- `autoCompile`: If true, automatically compiles the source before debugging
- `optLevel`: Optimization level (0-3). Use 0 for best debugging experience
- `args`: Array of command-line arguments
- `cwd`: Working directory for the program
- `env`: Object with environment variables

**VSCode Variables**:
- `${file}`: Current open file
- `${workspaceFolder}`: Workspace root directory
- `${fileBasename}`: Current file name
- `${fileDirname}`: Current file's directory

### Attach Configuration

Attaches to an already-running Vais process:

```json
{
  "type": "vais",
  "request": "attach",
  "name": "Attach to Process",
  "pid": 0,
  "program": "${workspaceFolder}/main",
  "stopOnAttach": true
}
```

**Options**:

- `pid`: Process ID to attach to. Use `${command:pickProcess}` to select interactively
- `program`: Path to the binary (for loading symbols)
- `stopOnAttach`: If true, pauses all threads when attaching

## Debugging Features

### Breakpoints

#### Line Breakpoints

1. Click in the gutter (left of line numbers) to set a breakpoint
2. Red dot appears to indicate the breakpoint
3. Breakpoints persist across debugging sessions

#### Conditional Breakpoints

Right-click on a breakpoint and select "Edit Breakpoint":

```
Expression: x > 10
Hit Count: 5
Log Message: Value of x is {x}
```

**Examples**:

```vais
# Break only when x is greater than 10
x > 10

# Break on the 5th hit
hit == 5

# Break when result is not null
result != null

# Complex condition
(counter > 0) && (flag == true)
```

#### Function Breakpoints

1. Open "Breakpoints" panel (Debug sidebar)
2. Click "+" in "Function Breakpoints" section
3. Enter function name (e.g., `main`, `calculate`, `processData`)

### Variable Inspection

#### Locals and Arguments

Variables are shown in the "Variables" panel when the program is paused:

```
▼ Locals
  x: 42 (i64)
  name: "hello" (String)
  ▼ point: Point
    x: 10 (i64)
    y: 20 (i64)
▼ Arguments
  arg1: 5 (i64)
  arg2: 10 (i64)
▼ Registers
  rax: 0x0000000000000042
  rbx: 0x0000000000000000
  ...
```

#### Modifying Variables

1. Right-click on a variable
2. Select "Set Value"
3. Enter the new value
4. Press Enter

#### Hovering

Hover over any variable in the editor to see its current value and type.

#### Watch Expressions

Add expressions to the "Watch" panel to track their values:

```
x + y
myArray[0]
calculate(x)
```

### Call Stack

The "Call Stack" panel shows the current execution stack:

```
▶ main (main.vais:10)
  ▶ process (main.vais:25)
    ▶ calculate (main.vais:42)
```

Click on any frame to:
- Navigate to that location in the code
- Inspect variables in that scope
- Evaluate expressions in that context

### Stepping

Use the debug toolbar or keyboard shortcuts:

- **Continue** (`F5`): Resume execution until the next breakpoint
- **Step Over** (`F10`): Execute the current line and stop at the next line
- **Step Into** (`F11`): Step into function calls
- **Step Out** (`Shift+F11`): Step out of the current function
- **Restart** (`Ctrl+Shift+F5`): Restart the debugging session
- **Stop** (`Shift+F5`): Stop debugging

### Debug Console

Use the Debug Console to:

#### Evaluate Expressions

```
> x + y
42
> calculate(10)
100
> myArray[0]
"first"
```

#### Execute Statements (REPL)

```
> x := 42
> print(x)
42
```

### Memory View

Right-click on a variable and select "View Memory" to inspect raw memory:

```
0x00007fff5fbff820: 42 00 00 00 00 00 00 00
0x00007fff5fbff828: 00 00 00 00 00 00 00 00
```

### Disassembly View

Right-click in the editor and select "Open Disassembly View" to see the assembly code:

```
0x100003f40: pushq %rbp
0x100003f41: movq %rsp, %rbp
0x100003f44: movl $42, -4(%rbp)
```

## Advanced Usage

### Debugging Tests

Create a separate configuration for testing:

```json
{
  "type": "vais",
  "request": "launch",
  "name": "Debug Tests",
  "program": "${workspaceFolder}/tests/test_main.vais",
  "stopOnEntry": false,
  "autoCompile": true
}
```

### Debugging with Arguments

```json
{
  "type": "vais",
  "request": "launch",
  "name": "Debug with Args",
  "program": "${workspaceFolder}/main.vais",
  "args": ["--input", "data.txt", "--verbose"],
  "stopOnEntry": false
}
```

### Debugging Pre-compiled Binaries

For faster iteration, compile separately and debug the binary:

```bash
vaisc main.vais -o main -g -O0
```

```json
{
  "type": "vais",
  "request": "launch",
  "name": "Debug Binary",
  "program": "${workspaceFolder}/main.vais",
  "binary": "${workspaceFolder}/main",
  "autoCompile": false,
  "stopOnEntry": true
}
```

### Debugging with Environment Variables

```json
{
  "type": "vais",
  "request": "launch",
  "name": "Debug with Env",
  "program": "${workspaceFolder}/main.vais",
  "env": {
    "LOG_LEVEL": "debug",
    "DATABASE_URL": "localhost:5432"
  }
}
```

### Multi-root Workspace

Each workspace folder can have its own launch.json:

```
project/
├── frontend/
│   └── .vscode/
│       └── launch.json  (frontend configs)
├── backend/
│   └── .vscode/
│       └── launch.json  (backend configs)
```

## Troubleshooting

### DAP Server Not Found

**Error**: "Cannot find vais-dap"

**Solution**:
1. Install the DAP server:
   ```bash
   cargo install --path crates/vais-dap
   ```
2. Or configure the path:
   ```json
   {
     "vais.debugAdapter.path": "/path/to/vais-dap"
   }
   ```

### LLDB Not Found

**Error**: "Failed to start LLDB"

**Solution**:
- macOS: Install Xcode Command Line Tools
  ```bash
  xcode-select --install
  ```
- Linux: Install LLDB
  ```bash
  # Ubuntu/Debian
  sudo apt-get install lldb

  # Fedora
  sudo dnf install lldb

  # Arch
  sudo pacman -S lldb
  ```

### No Debug Info

**Error**: "Debug symbols not found"

**Solution**:
Ensure the program is compiled with debug info:
```bash
vaisc main.vais -g
```

In launch.json, set:
```json
{
  "optLevel": 0  // Disable optimizations for better debugging
}
```

### Breakpoint Not Hit

**Possible causes**:

1. **Optimization**: Code may be inlined or optimized away
   - Solution: Use `optLevel: 0`

2. **Dead Code**: The line is never executed
   - Solution: Check program logic

3. **Wrong File**: Debugging a different version of the code
   - Solution: Ensure `autoCompile: true` or recompile manually

### Variables Not Visible

**Possible causes**:

1. **Optimized Out**: Variable eliminated by the optimizer
   - Solution: Use `optLevel: 0`

2. **Out of Scope**: Variable not in the current stack frame
   - Solution: Navigate to the correct stack frame

### Slow Step Performance

**Cause**: Large struct expansion or many variables

**Solution**:
- Collapse large structs in the Variables panel
- Use conditional breakpoints to skip iterations
- Set `"vais.trace.server": "off"` to reduce logging

## Best Practices

### 1. Use Optimization Level 0 for Debugging

```json
{
  "optLevel": 0
}
```

This preserves all variables and prevents code reordering.

### 2. Set Meaningful Breakpoint Conditions

Instead of:
```vais
# Breaking on every iteration
L i := 0; i < 1000; i := i + 1 {
    # Some code
}
```

Use:
```vais
# Conditional breakpoint: i == 999
L i := 0; i < 1000; i := i + 1 {
    # Some code  # <- Breakpoint with condition
}
```

### 3. Use Logpoints for Non-Intrusive Debugging

Right-click on a breakpoint → "Edit Breakpoint" → "Log Message":
```
Iteration {i}: value = {value}
```

This prints without stopping execution.

### 4. Save Debug Configurations

Create configurations for common debugging scenarios:
- Debug main program
- Debug tests
- Debug with specific args
- Attach to running process

### 5. Use the Debug Console

Quick expression evaluation without modifying code:
```
> calculate(x)
> myArray.length()
> debug_state()
```

## Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|---------------|-------|
| Start/Continue | `F5` | `F5` |
| Step Over | `F10` | `F10` |
| Step Into | `F11` | `F11` |
| Step Out | `Shift+F11` | `Shift+F11` |
| Restart | `Ctrl+Shift+F5` | `Cmd+Shift+F5` |
| Stop | `Shift+F5` | `Shift+F5` |
| Toggle Breakpoint | `F9` | `F9` |
| Run to Cursor | `Ctrl+F10` | `Cmd+F10` |

## Further Resources

- [DAP Specification](https://microsoft.github.io/debug-adapter-protocol/)
- [VSCode Debugging Guide](https://code.visualstudio.com/docs/editor/debugging)
- [Vais Language Documentation](../../docs/)
- [LLDB Tutorial](https://lldb.llvm.org/use/tutorial.html)

## Support

For issues or questions:
- File a GitHub issue: https://github.com/vaislang/vais/issues
- Check existing issues for solutions
- Include debug logs (set `"vais.trace.server": "verbose"`)
