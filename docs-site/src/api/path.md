# Path

File path manipulation utilities.

**Module:** `std/path.vais`

## Types

### `Path`

Immutable path reference with methods for inspection and manipulation.

```vais
S Path {
    raw: str,
}
```

### `PathBuf`

Owned, mutable path buffer for building paths.

```vais
S PathBuf {
    buf: str,
}
```

## Path Methods

### `join(other: str) -> str`

Joins two path components with the platform separator.

```vais
p := Path { raw: "/home/user" }
result := p.join("docs")  # "/home/user/docs"
```

### `parent() -> str`

Returns the parent directory.

```vais
p := Path { raw: "/home/user/file.txt" }
dir := p.parent()  # "/home/user"
```

### `filename() -> str`

Returns the file name component.

```vais
p := Path { raw: "/home/user/file.txt" }
name := p.filename()  # "file.txt"
```

### `extension() -> str`

Returns the file extension.

```vais
p := Path { raw: "main.vais" }
ext := p.extension()  # "vais"
```

### `stem() -> str`

Returns the file name without extension.

```vais
p := Path { raw: "main.vais" }
s := p.stem()  # "main"
```

### `is_absolute() -> bool`

Returns `true` if the path is absolute.

```vais
p := Path { raw: "/usr/bin" }
p.is_absolute()  # true
```

## Standalone Functions

### `path_join(a: str, b: str) -> str`

Joins two paths.

### `path_parent(p: str) -> str`

Returns parent directory of a path string.

### `path_filename(p: str) -> str`

Returns file name from a path string.

### `path_extension(p: str) -> str`

Returns extension from a path string.

## See Also

- [File](./file.md) — file I/O operations
- [Filesystem](./filesystem.md) — directory operations
