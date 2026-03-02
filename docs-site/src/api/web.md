# Web API Reference

> Browser API bindings for WebAssembly (DOM, console, timers, fetch, storage)

## Import

```vais
U std/web
```

## Overview

The `web` module provides browser API bindings for Vais programs compiled to WebAssembly. It exposes DOM manipulation, console output, timers, fetch, and local storage through `#[wasm_import("env", ...)]` declarations.

**Note:** This module requires the WASM target (`--target wasm32-unknown-unknown`) and host-provided JavaScript imports.

**Safety:** DOM handles (`i64`) are opaque references managed by the JavaScript host. Invalid handles may cause JavaScript exceptions.

## Console API

```vais
F log_str(msg: str) -> i64          # Write to browser console
F console_warn(msg: str) -> i64     # Console warning
F console_error(msg: str) -> i64    # Console error
```

## DOM API

```vais
F get_element_by_id(id: str) -> i64         # Get element by ID
F set_text_content(elem: i64, text: str)    # Set element text
F set_inner_html(elem: i64, html: str)      # Set element innerHTML
F create_element(tag: str) -> i64           # Create new element
F append_child(parent: i64, child: i64)     # Append child element
F set_attribute(elem: i64, name: str, value: str)  # Set attribute
F get_attribute(elem: i64, name: str) -> str        # Get attribute
F add_event_listener(elem: i64, event: str, handler: i64)  # Add event listener
```

## Timer API

```vais
F set_timeout(handler: i64, ms: i64) -> i64     # Schedule callback
F set_interval(handler: i64, ms: i64) -> i64    # Schedule repeating callback
F clear_timeout(id: i64)                          # Cancel timeout
F clear_interval(id: i64)                         # Cancel interval
```

## Fetch API

```vais
F fetch(url: str, callback: i64) -> i64    # HTTP fetch with callback
```

## Storage API

```vais
F local_storage_get(key: str) -> str       # Get from localStorage
F local_storage_set(key: str, value: str)  # Set in localStorage
F local_storage_remove(key: str)           # Remove from localStorage
```

## Example

```vais
U std/web

F main() {
    log_str("Hello from Vais!")
    elem := get_element_by_id("app")
    set_text_content(elem, "Welcome to Vais WASM")
    set_attribute(elem, "class", "active")
}
```
