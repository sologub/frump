# Frump Usage Guide

Complete reference for using the Frump task management CLI.

## Table of Contents

- [Installation](#installation)
- [Getting Started](#getting-started)
- [Core Commands](#core-commands)
- [Task Management](#task-management)
- [Filtering and Search](#filtering-and-search)
- [Git Integration](#git-integration)
- [Templates](#templates)
- [Bulk Operations](#bulk-operations)
- [Import/Export](#importexport)
- [Conflict Resolution](#conflict-resolution)
- [Examples and Workflows](#examples-and-workflows)

## Installation

```bash
cargo build --release
# Binary will be at target/release/frump
```

## Getting Started

Frump works with a `frump.md` file in your project directory. This file contains:
- A header section
- A team section listing team members
- A tasks section with all active tasks

### Basic frump.md Structure

```markdown
# My Project

Project description goes here.

## Team

* John Doe <john@example.com> - Lead Developer
* Jane Smith <jane@example.com> - QA Engineer

## Tasks

### Task 1 - Implement user authentication

Add login and registration functionality.
Status: working
Assigned To: John Doe

### Bug 2 - Fix navigation bug

The menu doesn't close on mobile.
Status: open
Assigned To: Jane Smith
```

## Core Commands

### list - List all tasks

List all tasks in the current frump.md file.

```bash
# List all tasks
frump list

# Filter by task type
frump list --task-type Bug
frump list -t Feature

# Filter by status
frump list --status working
frump list -s done

# Filter by assignee
frump list --assignee "John Doe"
frump list -a "Jane Smith"

# Combine filters
frump list -t Bug -s open -a "Jane Smith"
```

**Example output:**
```
Task 1 - Implement user authentication
  Status: working
  Assigned to: John Doe

Bug 2 - Fix navigation bug
  Status: open
  Assigned to: Jane Smith
```

### show - Show task details

Display complete information about a specific task.

```bash
frump show <task_id>
```

**Example:**
```bash
$ frump show 1

### Task 1 - Implement user authentication

Add login and registration functionality.
Authentication should support email/password and OAuth.

Status: working
Assigned To: John Doe
Priority: high
```

### add - Add a new task

Create a new task in frump.md.

```bash
# Simple task (defaults to type "Task")
frump add "Fix typo in README"

# Specify task type
frump add -t Bug "Login button not working"
frump add -t Feature "Add dark mode support"

# Add with body text
frump add "Refactor database code" -b "Current code is hard to maintain"

# Assign to someone
frump add "Write tests" -a "Jane Smith"

# Set status
frump add "Deploy to staging" -s "ready"

# Combine all options
frump add -t Feature "Add export feature" \
  -b "Users want to export their data as CSV" \
  -a "John Doe" \
  -s "planning"
```

**Note:** The add command automatically:
- Finds the next available task ID
- Checks git history to avoid ID conflicts
- Assigns to the first team member if no assignee is specified

## Task Management

### close - Close a task

Remove a task from frump.md (marks it as done/closed).

```bash
frump close <task_id>
```

**Example:**
```bash
$ frump close 5
Closed Task 5 - Fix typo in README

Remember to commit this change with a descriptive message.
```

**Tip:** Closed tasks remain in git history and can be viewed with `frump closed`.

### assign - Assign a task

Change or set the assignee for a task.

```bash
frump assign <task_id> <assignee_name>
```

**Examples:**
```bash
frump assign 3 "John Doe"
frump assign 7 "Jane Smith"
```

### set - Set a property

Set or update any property on a task.

```bash
frump set <task_id> <property_name> <value>
```

**Examples:**
```bash
# Set status
frump set 1 Status working
frump set 5 Status done

# Set priority
frump set 2 Priority high

# Set custom property (must be Capitalized, max 3 words)
frump set 3 "Due Date" "2025-12-31"
frump set 4 "Estimated Hours" 8
```

**Property Rules:**
- Must start with an uppercase letter
- Maximum 3 words
- Common properties: Status, Priority, Tags, "Assigned To", "Due Date"

### update - Update task content

Modify a task's subject or body.

```bash
# Update subject only
frump update <task_id> --subject "New subject text"

# Update body only
frump update <task_id> --body "New detailed description"

# Update both
frump update <task_id> \
  --subject "Updated title" \
  --body "Updated description"
```

**Example:**
```bash
$ frump update 3 --subject "Implement OAuth authentication"
Updated subject for task 3
```

## Filtering and Search

### search - Search tasks

Search for tasks by keyword in subject or body.

```bash
# Search in subjects only
frump search "authentication"

# Search in both subject and body
frump search "authentication" --full
frump search "bug" -f
```

**Example output:**
```
Found 2 task(s) matching 'auth':

Task 1 - Implement user authentication
Feature 8 - Add OAuth support
```

### stats - Show statistics

Display summary statistics about your tasks.

```bash
frump stats
```

**Example output:**
```
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
  (no status): 1

By Assignee:
  John Doe: 7
  Jane Smith: 5

Closed tasks: 8
```

### validate - Validate frump.md

Check your frump.md file for issues.

```bash
frump validate
```

**Checks performed:**
- File structure is valid
- All task IDs are unique
- Task IDs are sequential (warns about gaps)
- Team member emails are valid

**Example output:**
```
✓ File structure is valid
✓ All task IDs are unique
⚠ ID gaps found (possibly closed tasks):
  ID 3
  IDs 5-7
✓ Validation complete: 10 tasks, 3 team members
```

## Git Integration

Frump integrates with git to provide history tracking and prevent ID conflicts.

### history - Show task history

View the complete history of a task from git commits.

```bash
frump history <task_id>
```

**Example output:**
```
History for Task 5:

✓ Created by John Doe on 2025-11-20 14:30
  Commit: a3f8b901
  Message: Add authentication task

• Modified by Jane Smith on 2025-11-21 09:15
  Commit: c2e4d678
  Message: Update task priority

✗ Deleted by John Doe on 2025-11-24 16:45
  Commit: 9f2b3c45
  Message: Close completed authentication task
```

**Note:** Requires being in a git repository.

### closed - List closed tasks

Show all tasks that have been removed from frump.md but exist in history.

```bash
frump closed
```

**Example output:**
```
Closed tasks:

Task 3 - Initial project setup
Bug 5 - Fix login validation
Task 7 - Write documentation

Total: 3 closed tasks
```

## Templates

Templates help you quickly create tasks with predefined structure.

### template add - Create a template

```bash
frump template add <name> <subject_template> \
  -t <task_type> \
  -b <body_template>
```

Use `{placeholder}` syntax for variables.

**Examples:**
```bash
# Bug report template
frump template add bug "Fix {component} issue" \
  -t Bug \
  -b "Issue found in {component}: {description}"

# Feature template
frump template add feature "Add {feature} support" \
  -t Feature \
  -b "Users requested: {description}"

# Task template
frump template add task "{action} the {component}"
```

### template list - List templates

```bash
frump template list
```

**Example output:**
```
Available templates:

bug (Bug)
  Subject: Fix {component} issue
  Body: Issue found in {component}: {description}

feature (Feature)
  Subject: Add {feature} support
  Body: Users requested: {description}
```

### template show - Show template details

```bash
frump template show <name>
```

### template remove - Delete a template

```bash
frump template remove <name>
```

## Bulk Operations

Perform operations on multiple tasks at once.

### bulk close-by-status - Close tasks by status

Close all tasks matching a specific status.

```bash
frump bulk close-by-status <status>
```

**Example:**
```bash
$ frump bulk close-by-status done
Closed 5 task(s) with status 'done'
```

### bulk assign-by-type - Assign tasks by type

Assign all tasks of a specific type to someone.

```bash
frump bulk assign-by-type <task_type> <assignee>
```

**Example:**
```bash
$ frump bulk assign-by-type Bug "Jane Smith"
Assigned 7 task(s) of type 'Bug' to Jane Smith
```

### bulk set-by-status - Set property by status

Set a property on all tasks with a specific status.

```bash
frump bulk set-by-status <status> <property> <value>
```

**Example:**
```bash
$ frump bulk set-by-status working Priority high
Set Priority = high on 3 task(s) with status 'working'
```

## Import/Export

### export - Export tasks

Export tasks to JSON or CSV format.

```bash
# Export to JSON (stdout)
frump export

# Export to JSON file
frump export -o tasks.json

# Export to CSV
frump export --format csv -o tasks.csv
frump export -f csv -o tasks.csv
```

**JSON format** includes:
- Header text
- Team members with emails and roles
- All tasks with full details

**CSV format** includes:
- One row per task
- Columns: ID, Type, Subject, Body, Status, Assignee

### import - Import tasks

Import tasks from a JSON file.

```bash
# Replace all tasks (careful!)
frump import tasks.json

# Merge with existing tasks (adds new IDs)
frump import tasks.json --merge
frump import tasks.json -m
```

**Note:** When merging, imported tasks get new IDs to avoid conflicts.

## Conflict Resolution

When merging git branches, task IDs may conflict if both branches added tasks with the same ID.

### check-conflicts - Detect conflicts

Check for duplicate task IDs.

```bash
frump check-conflicts
```

**Example output with conflicts:**
```
✗ Found 2 duplicate task ID(s):

ID 5:
  - Task 5: Add user profile page
  - Feature 5: Implement notifications

ID 8:
  - Bug 8: Fix memory leak
  - Task 8: Update dependencies

Run 'frump resolve-conflicts' to automatically renumber conflicts
```

**Example output without conflicts:**
```
✓ No duplicate task IDs found
✓ File is ready for merge
```

### resolve-conflicts - Fix conflicts

Automatically renumber conflicting tasks.

```bash
# Resolve and save changes
frump resolve-conflicts

# Resolve and commit automatically
frump resolve-conflicts --commit
frump resolve-conflicts -c
```

**Example:**
```bash
$ frump resolve-conflicts --commit
✓ Resolved 2 duplicate task ID(s):

  5 → 12: Implement notifications
  8 → 13: Update dependencies

✓ Changes committed automatically
```

**How it works:**
- Keeps the first occurrence of each duplicate ID
- Renumbers subsequent duplicates to the next available IDs
- Optionally creates a git commit with the changes

## Examples and Workflows

### Daily Development Workflow

```bash
# Start your day - see what you're working on
frump list -a "Your Name" -s working

# Add a new task you discovered
frump add -t Bug "Login timeout not working" -b "Users get logged out too quickly"

# Update task status as you work
frump set 5 Status working

# Close completed tasks
frump close 3
git add frump.md
git commit -m "Close task 3: Completed user authentication"

# End of day - see what's left
frump stats
```

### Code Review Workflow

```bash
# Reviewer adds issues found
frump add -t Bug "Missing null check in login" -a "Developer Name"
frump add -t Task "Add tests for edge cases" -a "Developer Name"

# Developer fixes and tracks progress
frump list -a "Developer Name" -s open
frump set 10 Status working
# ... fix the issue ...
frump close 10
```

### Team Handoff Workflow

```bash
# Reassign all your tasks to someone else
frump list -a "Your Name"  # See what you have
frump bulk assign-by-type Task "Other Person"

# Or reassign specific task
frump assign 7 "Other Person"
```

### Release Planning Workflow

```bash
# See all features planned
frump list -t Feature

# Mark features as ready for next release
frump bulk set-by-status done "Release" "v2.0"

# Export for external tracking
frump export -o v2.0-tasks.json
```

### Merge Conflict Resolution

```bash
# After merging branches
frump check-conflicts  # Check for ID conflicts

# If conflicts found
frump resolve-conflicts --commit

# Verify everything is clean
frump validate
frump list
```

## Tips and Best Practices

1. **Commit Often**: Commit frump.md changes regularly so you can track task history.

2. **Use Descriptive Commit Messages**: Your commit messages become part of task history.
   ```bash
   git commit -m "Add task 15: Implement dark mode"
   git commit -m "Update task 7: Change priority to high"
   git commit -m "Close task 12: Feature completed and tested"
   ```

3. **Leverage Filtering**: Use filters to focus on relevant tasks.
   ```bash
   frump list -s working  # What's in progress
   frump list -t Bug      # All bugs
   ```

4. **Use Templates**: Create templates for common task types to ensure consistency.

5. **Regular Validation**: Run `frump validate` periodically to catch issues early.

6. **Close Tasks Promptly**: Close tasks when done to keep your list manageable.
   ```bash
   frump close 5
   ```

7. **Check History**: Use `frump history` to understand how a task evolved.

8. **Export for Backup**: Periodically export to JSON for backup.
   ```bash
   frump export -o backup-$(date +%Y%m%d).json
   ```

## File Location

By default, frump looks for `frump.md` in the current directory. You can specify a different file:

```bash
frump --file path/to/custom.md list
frump -f docs/tasks.md show 5
```

## Getting Help

```bash
# General help
frump --help

# Command-specific help
frump list --help
frump add --help
frump bulk --help
```

## Troubleshooting

### "Task ID must be positive"
- Task IDs start at 1, not 0
- This usually means a parsing error

### "Property key must start with uppercase letter"
- Property names must be capitalized: `Status`, not `status`
- Max 3 words: `Due Date` ✓, `Expected Completion Date` ✗

### "Not in a git repository"
- Some commands (history, closed, git-aware add) require git
- Initialize git: `git init`

### "Failed to read frump.md file"
- Make sure frump.md exists in your directory
- Or specify the path: `frump --file path/to/frump.md`

### Duplicate IDs after merge
- Run `frump check-conflicts`
- Run `frump resolve-conflicts --commit`

## See Also

- [README.md](README.md) - Project overview and concepts
- [PLAN.md](PLAN.md) - Implementation roadmap and architecture
