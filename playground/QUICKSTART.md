# Vais Playground - Quick Start

Get the playground running in under 2 minutes!

## Prerequisites

- Node.js 18+ ([download](https://nodejs.org/))
- npm (comes with Node.js)
- Modern web browser (Chrome, Firefox, Safari, Edge)

## Installation

```bash
# Navigate to playground directory
cd playground

# Install dependencies (first time only)
npm install

# Start development server
npm run dev
```

The playground will automatically open at `http://localhost:3000`

## Using the Playground

### 1. Select an Example

Click any example in the left sidebar:
- Hello World
- Fibonacci
- Generics
- And 10 more...

### 2. Edit Code

The Monaco editor supports:
- Syntax highlighting
- Auto-completion (Ctrl+Space)
- Code folding
- Multiple cursors

### 3. Run Code

Click the "Run" button or press **Ctrl+Enter**

### 4. View Output

Results appear in the right panel with:
- Compilation status
- Program output
- Error messages (if any)

## Quick Tips

| Action | Shortcut |
|--------|----------|
| Run code | `Ctrl/Cmd + Enter` |
| Format code | `Ctrl/Cmd + S` |
| Auto-complete | `Ctrl + Space` |
| Find | `Ctrl/Cmd + F` |

## Example Code

```vais
# Hello World
F main()->i64 {
    puts("Hello, Vais!")
    0
}
```

```vais
# Fibonacci with self-recursion
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)
F main()->i64 = fib(10)
```

## Next Steps

- 📖 Read the [Tutorial](TUTORIAL.md) for step-by-step learning
- 🎯 Check [Features](FEATURES.md) for complete feature list
- 🚀 See [Deployment](DEPLOYMENT.md) for hosting options
- 🔧 Read [Integration](INTEGRATION.md) for WASM setup

## Troubleshooting

### Port already in use

```bash
# Use a different port
npm run dev -- --port 3001
```

### Dependencies not installing

```bash
# Clear cache and reinstall
rm -rf node_modules package-lock.json
npm install
```

### Browser not opening

Manually navigate to `http://localhost:3000`

## Building for Production

```bash
# Create optimized build
npm run build

# Preview production build
npm run preview
```

Output is in the `dist/` directory.

## Alternative: Quick Start Script

```bash
# Use the provided script
./start.sh

# Or with WASM build (requires Rust)
./start.sh --with-wasm
```

## Support

- 🐛 [Report Issues](https://github.com/sswoo88/vais/issues)
- 💬 [Discussions](https://github.com/sswoo88/vais/discussions)
- 📚 [Full Documentation](https://github.com/sswoo88/vais)

---

Ready to code? Run `npm run dev` and start exploring Vais! ⚡
