# Frump

Distributed task management tool based on Git and Markdown. Manage tasks alongside your code with full version history.

## Why Frump?

**Problem**: Traditional task management tools keep tasks separate from code, making it hard to see the full project picture from git history alone. Teams also lack truly distributed task collaboration.

**Solution**: Frump stores tasks as Markdown in your git repository. When you clone a repo, you get code, current tasks, and complete task history. Tasks can be branched, merged, and versioned just like code.

## Philosophy

**No tools required**: Frump uses simple conventions in a `frump.md` file. You can read and edit tasks with any text editor.

**Git-native**: Tasks live in git alongside code. `git pull` gets new tasks, `git push` shares your changes. Task history is preserved in git commits.

**Simple & readable**: Everything is plain Markdown. No databases, no servers, just files and git.

## Quick Start

### Installation

```bash
# Build from source
cargo build --release

# Binary will be at target/release/frump
# Optionally copy to your PATH
cp target/release/frump /usr/local/bin/
```

### Create your first project

```bash
cd my-project
git init

# Create initial frump.md
cat > frump.md << 'EOF'
# My Project

## Team

* Your Name <you@example.com> - Lead Developer

## Tasks
EOF

git add frump.md
git commit -m "Initialize frump"
```

### Basic commands

```bash
# Add a task
frump add "Implement user authentication"

# List all tasks
frump list

# View task details
frump show 1

# Mark task as working
frump set 1 Status working

# Close completed task
frump close 1
git commit -am "Complete task 1"
```

## CLI Reference

### Task Management

```bash
# Add tasks
frump add "Task subject"
frump add -t Bug "Fix login issue" -b "Details here"
frump add "New feature" -a "John Doe" -s working

# List and filter
frump list                          # All tasks
frump list -t Bug                   # Only bugs
frump list -s working               # Tasks with status "working"
frump list -a "John Doe"           # Assigned to John

# View and modify
frump show 5                        # Show task details
frump assign 5 "Jane Smith"        # Reassign task
frump set 5 Priority high          # Set property
frump update 5 --subject "New title"  # Update subject
frump close 5                       # Close task
```

### Search & Analysis

```bash
frump search "authentication"      # Search in subjects
frump search "login" --full        # Search subjects and bodies
frump stats                        # Show statistics
frump validate                     # Check file integrity
```

### Git Integration

```bash
frump history 5                    # Show task lifecycle from git
frump closed                       # List all closed tasks
```

### Advanced Features

```bash
# Templates
frump template add bug "Fix {component} issue" -t Bug
frump template list

# Bulk operations
frump bulk close-by-status done
frump bulk assign-by-type Bug "QA Team"

# Import/Export
frump export -o backup.json
frump export -f csv -o tasks.csv
frump import backup.json --merge
```

### Conflict Resolution

When merging branches with conflicting task IDs:

```bash
frump check-conflicts             # Detect duplicates
frump resolve-conflicts --commit  # Auto-fix and commit
```

## Format Specification

### File Structure

```markdown
# Project Title

Brief description of the project.

## Team

* John Doe <john@example.com> - Lead Developer
* Jane Smith <jane@example.com> - QA Engineer

## Tasks

### Task 1 - Implement authentication

Add login and registration with email/password.

Status: working
Assigned To: John Doe
Priority: high

### Bug 2 - Fix navigation

The menu doesn't close properly on mobile devices.

Status: open
Assigned To: Jane Smith
```

### Task Format

```
### <Type> <ID> - <Subject>

<Body text - optional, multiple lines>

<Property>: <Value>
<Property>: <Value>
```

**Task ID**: Positive integer, unique across all history. CLI automatically assigns next available ID.

**Task Type**: Task, Bug, Issue, Feature, or custom. Default: Task.

**Subject**: Brief description (not a title, don't capitalize like a title).

**Properties**: Name-value pairs. Names must be Capitalized and max 3 words. Common properties:
- `Status`: open, working, review, done, blocked
- `Assigned To`: Team member name
- `Priority`: low, medium, high
- `Tags`: Comma-separated tags
- `Due Date`: Target date

## Common Workflows

### Daily Development

```bash
# Morning: check your tasks
frump list -a "Your Name" -s working

# Found a bug while coding
frump add -t Bug "Null pointer in profile page"

# Update task as you work
frump set 5 Status working

# Complete task
frump close 5
git add frump.md
git commit -m "Complete task 5: Add user profile"
```

### Code Review

```bash
# Reviewer adds issues
frump add -t Bug "Missing error handling" -a "Developer"

# Developer addresses issue
frump set 10 Status working
# ... make fixes ...
frump close 10
git commit -am "Fix task 10: Add error handling"
```

### Release Planning

```bash
# See all planned features
frump list -t Feature

# Mark done features for release
frump bulk set-by-status done Release v2.0

# Export release tasks
frump export -o release-v2.0.json
```

### Merge Conflict Resolution

```bash
# After merging branches
git merge feature/new-feature

# Check for ID conflicts
frump check-conflicts
# ✗ Found 2 duplicate task ID(s)

# Auto-resolve
frump resolve-conflicts --commit
# ✓ Resolved 2 duplicate task ID(s):
#   5 → 12: New feature
#   8 → 13: Bug fix
```

## Features

### ✅ Implemented

- **Strong typing** with validation (TaskId, PropertyKey, Email)
- **20+ commands** for complete task management
- **Git integration** with history tracking and conflict prevention
- **Smart ID assignment** that checks git history
- **Templates** for reusable task patterns
- **Bulk operations** for batch modifications
- **Import/Export** (JSON and CSV formats)
- **Conflict resolution** for merge scenarios
- **Property-based testing** with 60+ tests passing
- **Comprehensive docs** (USAGE.md, TUTORIAL.md)

### Architecture

```
src/
├── domain/        # Core types (Task, Team, Properties)
├── parser/        # Markdown parsing/serialization
├── git/           # Git history integration
├── export/        # JSON/CSV import/export
└── templates/     # Task template system
```

**Type system**: Validated newtypes prevent invalid states at compile time.

**Testing**: 56 unit tests + 4 doctests with property-based round-trip testing.

## Documentation

- **[USAGE.md](USAGE.md)**: Complete command reference with examples
- **[TUTORIAL.md](TUTORIAL.md)**: Step-by-step walkthrough for new users
- **[PLAN.md](PLAN.md)**: Implementation roadmap and architecture details

## Examples

### Statistics Output

```
$ frump stats
Task Statistics

Total tasks: 12

By Type:
  Task: 7
  Bug: 3
  Feature: 2

By Status:
  working: 5
  open: 4
  done: 2

By Assignee:
  John Doe: 7
  Jane Smith: 5

Closed tasks: 8
```

### Task History

```
$ frump history 5
History for Task 5:

✓ Created by John Doe on 2025-11-24 10:00
  Commit: a3f8b901
  Message: Add authentication task

• Modified by Jane Smith on 2025-11-24 11:30
  Commit: c2e4d678
  Message: Update priority to high

✗ Deleted by John Doe on 2025-11-24 15:45
  Commit: 9f2b3c45
  Message: Complete authentication feature
```

## Best Practices

1. **Commit frequently**: Every frump.md change should be committed with a clear message
2. **Descriptive subjects**: "Fix memory leak in WebSocket handler" not "Fix bug"
3. **Add context**: Use task body for implementation details and requirements
4. **Use filters**: `frump list -s working` to focus on active tasks
5. **Close promptly**: Close tasks when done to keep the list clean
6. **Check history**: Use `frump history` to understand task evolution

## Why "frump"?

Because with git they make a lovely couple.

## Contributing

See [PLAN.md](PLAN.md) for architecture details. All 8 implementation phases are complete.

## License

MIT
