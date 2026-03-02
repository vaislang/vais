# Process API Reference

> Process execution operations via C standard library

## Import

```vais
U std/process
```

## Overview

The `process` module provides functions for spawning and managing external processes. It wraps the C standard library functions `system`, `popen`, `pclose`, and `exit`.

## Functions

### process_run

```vais
F process_run(command: str) -> i32
```

Run a shell command and return its exit status.

**Parameters:**
- `command`: The shell command string to execute

**Returns:** The exit status (`0` = success).

### process_open

```vais
F process_open(command: str) -> i64
```

Run a command and open a pipe to read its output.

**Parameters:**
- `command`: The shell command to execute

**Returns:** A file handle (i64) for reading output, or `0` on error.

### process_close

```vais
F process_close(handle: i64) -> i32
```

Close a process handle opened with `process_open` and get the exit status.

**Parameters:**
- `handle`: The file handle returned by `process_open`

**Returns:** The exit status of the process.

### process_exit

```vais
F process_exit(status: i32)
```

Exit the current process with a status code.

**Parameters:**
- `status`: The exit code (0 = success)

## Example

```vais
U std/process

F main() {
    # Run a command
    status := process_run("echo hello")

    # Capture output
    handle := process_open("ls -la")
    process_close(handle)
}
```
