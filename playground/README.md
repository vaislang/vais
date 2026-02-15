# Vais Playground

Web-based interactive playground for the Vais programming language.

## Features

- **Monaco Editor** with Vais syntax highlighting
- **Real-time compilation** and execution (Server, WASM, or Preview mode)
- **18 example programs** demonstrating language features
- **Auto-completion** for Vais keywords and functions
- **Keyboard shortcuts** for quick actions
- **Responsive design** for desktop and mobile
- **Dark theme** optimized for code editing

## Quick Start

### Installation

```bash
cd playground
npm install
```

### Development

```bash
npm run dev
```

This will start a development server at `http://localhost:3000`

### Building for Production

```bash
npm run build
npm run preview
```

## Project Structure

```
playground/
├── index.html              # Main HTML file
├── src/
│   ├── main.js            # Application entry point
│   ├── styles.css         # Global styles
│   ├── vais-language.js   # Monaco language definition
│   ├── compiler.js        # Compiler interface
│   └── examples.js        # Example code snippets
├── package.json           # Dependencies
└── vite.config.js         # Vite configuration
```

## Examples Included

1. **Hello World** - Simple program with output
2. **Fibonacci** - Recursive function with self-recursion operator
3. **Generics** - Generic function example
4. **Control Flow** - If-else and loops
5. **Struct** - Struct definition and methods
6. **Enum** - Enum types and pattern matching
7. **Match** - Pattern matching expressions
8. **Loops** - Different loop types
9. **Self Recursion** - Using the @ operator
10. **Type Inference** - Automatic type inference
11. **Operators** - Arithmetic and logical operators
12. **Functions** - Function definitions
13. **String Interpolation** - String formatting with variables
14. **Pipe Operator** - Function chaining with |>
15. **Mutable Variables** - Mutable references with ~mut
16. **Destructuring** - Pattern destructuring
17. **Type Infer Params** - Parameter type inference
18. **Minimal** - Simplest valid program

## Keyboard Shortcuts

- `Ctrl/Cmd + Enter` - Run code
- `Ctrl/Cmd + S` - Format code

## Features

### Syntax Highlighting

The playground includes full syntax highlighting for Vais:
- Keywords: `F`, `S`, `E`, `I`, `L`, `M`, etc.
- Types: `i64`, `f64`, `bool`, etc.
- Operators: `@`, `:=`, `=>`, etc.
- Comments, strings, and numbers

### Auto-completion

Press `Ctrl+Space` to see suggestions for:
- Keywords and control structures
- Type annotations
- Built-in functions
- Code snippets

### 3-Tier Compilation

The playground uses a 3-tier execution model with automatic fallback:

1. **Server mode** — Sends code to the playground server for real compilation via `vaisc`
2. **WASM mode** — Compiles and runs code in-browser using WebAssembly
3. **Preview mode** — Client-side mock compiler for basic syntax validation and demonstration

## Development

### Adding New Examples

Edit `src/examples.js`:

```javascript
export const examples = {
  'my-example': {
    name: 'My Example',
    description: 'Example description',
    code: `# Your Vais code here
F main() -> i64 = 0`
  }
};
```

### Customizing the Theme

Edit the theme in `src/vais-language.js`:

```javascript
monaco.editor.defineTheme('vais-dark', {
  base: 'vs-dark',
  inherit: true,
  rules: [
    // Add custom token colors
  ]
});
```

### Modifying Styles

Edit `src/styles.css` to customize the appearance.

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Any modern browser with ES6+ and WebAssembly support

## Dependencies

- **monaco-editor** - VS Code's editor component
- **vite** - Fast development server and build tool

## Future Enhancements

- [x] Real WASM-based compilation
- [ ] Code sharing via URL
- [ ] Multi-file projects
- [ ] Standard library documentation integration
- [ ] Performance profiling
- [ ] Assembly output viewer
- [ ] Mobile-optimized UI
- [ ] Collaborative editing
- [ ] Custom themes

## License

MIT License - see LICENSE file for details
