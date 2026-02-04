# Vais TODO - Usage Examples

## Quick Start

```bash
# Build the project
./build.sh

# Or manually compile
vaisc src/main.vais -o vais-todo
```

## Basic Usage

### 1. Getting Help

```bash
$ vais-todo help

Vais TODO - Terminal TODO Manager

USAGE:
  vais-todo <command> [arguments]

COMMANDS:
  add <description>     Add a new TODO item
  done <id>             Mark item as completed
  remove <id>           Remove an item
  list                  Show all items
  list --done           Show only completed items
  list --pending        Show only pending items
  help                  Show this help message

EXAMPLES:
  vais-todo add "Buy groceries"
  vais-todo done 1
  vais-todo list
  vais-todo remove 2
```

### 2. Adding TODOs

```bash
# Add a simple task
$ vais-todo add "Write documentation"
✓ TODO item added!

# Add another task
$ vais-todo add "Review code changes"
✓ TODO item added!

# Add a task with special characters
$ vais-todo add "Fix bug #123 in parser module"
✓ TODO item added!
```

### 3. Viewing TODOs

```bash
# View all tasks
$ vais-todo list
TODO List:

[ ] 1. Write documentation
[ ] 2. Review code changes
[ ] 3. Fix bug #123 in parser module

─────────────────────────────
Total: 3  |  Pending: 3  |  Done: 0

# View only pending tasks
$ vais-todo list --pending
TODO List:

[ ] 1. Write documentation
[ ] 2. Review code changes
[ ] 3. Fix bug #123 in parser module

# View only completed tasks (when you have some)
$ vais-todo list --done
No items match the filter.
```

### 4. Completing Tasks

```bash
# Mark task #1 as done
$ vais-todo done 1
✓ TODO item marked as done!

# View the updated list
$ vais-todo list
TODO List:

[✓] 1. Write documentation
[ ] 2. Review code changes
[ ] 3. Fix bug #123 in parser module

─────────────────────────────
Total: 3  |  Pending: 2  |  Done: 1
```

### 5. Removing Tasks

```bash
# Remove task #2
$ vais-todo remove 2
✓ TODO item removed!

# View the updated list
$ vais-todo list
TODO List:

[✓] 1. Write documentation
[ ] 3. Fix bug #123 in parser module

─────────────────────────────
Total: 2  |  Pending: 1  |  Done: 1
```

## Advanced Workflows

### Morning Routine

```bash
# Check yesterday's progress
vais-todo list --done

# Review what's pending
vais-todo list --pending

# Add today's goals
vais-todo add "Complete Phase 34 Stage 6"
vais-todo add "Test CLI tool functionality"
vais-todo add "Write project documentation"
```

### End of Day Cleanup

```bash
# Mark completed items
vais-todo done 4
vais-todo done 5

# Remove items no longer needed
vais-todo remove 3

# View summary
vais-todo list
```

### Project Planning

```bash
# Add all tasks for a project
vais-todo add "Design system architecture"
vais-todo add "Implement core functionality"
vais-todo add "Write unit tests"
vais-todo add "Create integration tests"
vais-todo add "Write user documentation"
vais-todo add "Deploy to production"

# Track progress
vais-todo list
```

### Weekly Review

```bash
# See all completed tasks
vais-todo list --done

# Count remaining work
vais-todo list --pending

# Clean up old completed items
# (manually remove by ID)
```

## Color Legend

When viewing the TODO list, items are color-coded:

- **Green [✓]**: Completed tasks
- **Yellow [ ]**: Pending tasks
- **Blue**: Section headers (USAGE, COMMANDS, etc.)
- **Red**: Error messages
- **Gray**: Metadata (statistics, dividers)

## Data Storage

All TODO items are stored in `~/.vais-todo.json`:

```bash
# View the raw data file
cat ~/.vais-todo.json

# Backup your TODOs
cp ~/.vais-todo.json ~/vais-todo-backup.json

# Restore from backup
cp ~/vais-todo-backup.json ~/.vais-todo.json
```

## Error Handling

### Item Not Found

```bash
$ vais-todo done 999
Error: TODO item not found
```

### Missing Arguments

```bash
$ vais-todo add
Error: Missing description. Usage: add <description>

$ vais-todo done
Error: Missing ID. Usage: done <id>
```

### Unknown Command

```bash
$ vais-todo xyz
Error: Unknown command. Use 'help' to see available commands.
```

## Tips & Tricks

1. **Keep descriptions concise**: Aim for 5-10 words per task
2. **Use IDs efficiently**: Note the ID when adding tasks
3. **Regular cleanup**: Remove or complete old tasks weekly
4. **Batch operations**: Add multiple tasks at once during planning
5. **Filter views**: Use `--pending` to focus on current work

## Integration Ideas

```bash
# Add to shell alias
alias t='vais-todo'
alias tl='vais-todo list'
alias ta='vais-todo add'
alias td='vais-todo done'

# Use in scripts
#!/bin/bash
vais-todo add "Automated task from script"

# Morning standup script
echo "Today's tasks:"
vais-todo list --pending
```

## Comparison with Other Tools

| Feature | vais-todo | todo.txt | taskwarrior |
|---------|-----------|----------|-------------|
| Speed | ⚡ Fast | Fast | Medium |
| Colors | ✓ | Limited | ✓ |
| Storage | JSON | Plain text | Custom DB |
| Simplicity | ✓ | ✓ | Complex |
| Language | Vais | Shell | C++ |

## FAQ

**Q: Where are my TODOs stored?**
A: In `~/.vais-todo.json` in your home directory.

**Q: Can I sync across machines?**
A: Not yet, but you can manually copy the JSON file.

**Q: What if I delete the JSON file?**
A: A new empty list will be created on next use.

**Q: Can I edit the JSON file directly?**
A: Yes, but be careful with the format. Use the CLI for safety.

**Q: How many TODOs can I have?**
A: Currently limited by memory, but designed for hundreds of items.

## Troubleshooting

### Build Issues

```bash
# If vaisc is not found
export PATH=$PATH:/path/to/vais/target/release

# If compilation fails
cd /Users/sswoo/study/projects/vais
cargo build --release
```

### Runtime Issues

```bash
# If file permissions error
chmod 644 ~/.vais-todo.json

# If JSON is corrupted
rm ~/.vais-todo.json
vais-todo list  # Creates new file
```

## Contributing

Found a bug or have a feature request? The project structure is:

- `src/main.vais` - CLI entry point
- `src/todo.vais` - Data model
- `src/storage.vais` - Persistence
- `src/display.vais` - UI/output

Make changes and test with `./build.sh`.
