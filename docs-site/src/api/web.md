# Web API Reference

> Browser API bindings for WebAssembly (DOM, console, timers, fetch, storage)

> **Implementation:** WASM-only module. Requires `--target wasm32-unknown-unknown` compilation and a browser/JS host environment. Functions call JavaScript APIs via WASM imports.

## Import

```vais
use std/web
```

## Overview

The `web` module provides browser API bindings for Vais programs compiled to WebAssembly. It exposes DOM manipulation, console output, timers, fetch, and local storage through `#[wasm_import("env", ...)]` declarations.

**Note:** This module requires the WASM target (`--target wasm32-unknown-unknown`) and host-provided JavaScript imports.

**Safety:** DOM handles (`i64`) are opaque references managed by the JavaScript host. Invalid handles may cause JavaScript exceptions.

## Console API

```vais
fn log_str(msg: str) -> i64          # Write to browser console
fn console_warn(msg: str) -> i64     # Console warning
fn console_error(msg: str) -> i64    # Console error
```

## DOM API

```vais
fn get_element_by_id(id: str) -> i64         # Get element by ID
fn set_text_content(elem: i64, text: str)    # Set element text
fn set_inner_html(elem: i64, html: str)      # Set element innerHTML
fn create_element(tag: str) -> i64           # Create new element
fn append_child(parent: i64, child: i64)     # Append child element
fn set_attribute(elem: i64, name: str, value: str)  # Set attribute
fn get_attribute(elem: i64, name: str) -> str        # Get attribute
fn add_event_listener(elem: i64, event: str, handler: i64)  # Add event listener
```

## Timer API

```vais
fn set_timeout(handler: i64, ms: i64) -> i64     # Schedule callback
fn set_interval(handler: i64, ms: i64) -> i64    # Schedule repeating callback
fn clear_timeout(id: i64)                          # Cancel timeout
fn clear_interval(id: i64)                         # Cancel interval
```

## Fetch API

```vais
fn fetch(url: str, callback: i64) -> i64    # HTTP fetch with callback
```

## Storage API

```vais
fn local_storage_get(key: str) -> str       # Get from localStorage
fn local_storage_set(key: str, value: str)  # Set in localStorage
fn local_storage_remove(key: str)           # Remove from localStorage
```

## Example

```vais
use std/web

fn main() {
    log_str("Hello from Vais!")
    elem := get_element_by_id("app")
    set_text_content(elem, "Welcome to Vais WASM")
    set_attribute(elem, "class", "active")
}
```
