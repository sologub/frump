# Frump Tutorial: Getting Started

Welcome to Frump! This tutorial will walk you through setting up and using Frump for task management in your project.

## What You'll Learn

- Setting up a Frump project
- Creating and managing tasks
- Working with your team
- Using git integration
- Advanced features

## Prerequisites

- Basic command-line knowledge
- Git installed (for history features)
- Rust and Cargo installed (to build Frump)

## Part 1: Installation and Setup

### Step 1: Build Frump

```bash
# Clone the repository (if you haven't already)
git clone https://github.com/sologub/frump
cd frump

# Build the project
cargo build --release

# The binary is now at target/release/frump
# Optionally, add it to your PATH or create an alias
alias frump='path/to/frump/target/release/frump'
```

### Step 2: Create Your First Frump Project

Let's create a simple web application project:

```bash
# Create and enter your project directory
mkdir my-web-app
cd my-web-app

# Initialize git (important for Frump's history features)
git init

# Create a basic frump.md file
cat > frump.md << 'EOF'
# My Web Application

A simple web application for learning Frump.

## Team

* Alice Johnson <alice@example.com> - Lead Developer
* Bob Smith <bob@example.com> - Backend Developer

## Tasks
EOF

# Commit the initial file
git add frump.md
git commit -m "Initial frump.md setup"
```

### Step 3: Verify Everything Works

```bash
# List tasks (should be empty)
frump list

# Validate the file
frump validate
```

Expected output:
```
No tasks found.
```

```
✓ File structure is valid
✓ All task IDs are unique
✓ Task IDs are sequential
✓ Validation complete: 0 tasks, 2 team members
```

## Part 2: Creating Your First Tasks

### Step 4: Add Some Tasks

Let's add tasks for building our web app:

```bash
# Add a basic task (assigned to first team member by default)
frump add "Set up project structure"

# Add a feature with description
frump add -t Feature "Create user registration" \
  -b "Users should be able to register with email and password"

# Add a bug (even though we haven't started coding yet!)
frump add -t Bug "Fix login redirect issue" \
  -b "Users are redirected to wrong page after login" \
  -a "Bob Smith"

# Add a task with status
frump add "Write API documentation" \
  -s "planned" \
  -a "Alice Johnson"
```

### Step 5: View Your Tasks

```bash
# List all tasks
frump list
```

Expected output:
```
Task 1 - Set up project structure
  Assigned to: Alice Johnson

Feature 2 - Create user registration
  Assigned to: Alice Johnson

Bug 3 - Fix login redirect issue
  Assigned to: Bob Smith

Task 4 - Write API documentation
  Status: planned
  Assigned to: Alice Johnson
```

### Step 6: Commit Your Changes

Remember, Frump works best with git:

```bash
git add frump.md
git commit -m "Add initial project tasks"
```

## Part 3: Managing Tasks

### Step 7: Working on a Task

Let's say Alice starts working on task 1:

```bash
# Update the status
frump set 1 Status working

# Check the change
frump show 1
```

Output:
```
### Task 1 - Set up project structure

Status: working
Assigned To: Alice Johnson
```

### Step 8: Adding Details to a Task

Add more information as you work:

```bash
# Set priority
frump set 1 Priority high

# Set due date
frump set 1 "Due Date" "2025-12-01"

# Update the body with more details
frump update 1 --body "Set up project structure including:
- Initialize npm project
- Configure webpack
- Set up testing framework"

# View the updated task
frump show 1
```

### Step 9: Completing a Task

When you finish a task:

```bash
# Close the task
frump close 1

# Commit with a descriptive message
git add frump.md
git commit -m "Complete task 1: Project structure set up"
```

## Part 4: Filtering and Searching

### Step 10: Filter Tasks

```bash
# See only bugs
frump list -t Bug

# See what Alice is working on
frump list -a "Alice Johnson"

# See all tasks with status "working"
frump list -s working

# Combine filters: bugs assigned to Bob
frump list -t Bug -a "Bob Smith"
```

### Step 11: Search for Tasks

```bash
# Search in task subjects
frump search "registration"

# Search in both subject and body
frump search "login" --full
```

### Step 12: Get Statistics

```bash
# See project overview
frump stats
```

Output:
```
Task Statistics

Total tasks: 3

By Type:
  Feature: 1
  Bug: 1
  Task: 1

By Status:
  working: 1
  (no status): 2

By Assignee:
  Alice Johnson: 2
  Bob Smith: 1
```

## Part 5: Git Integration

### Step 13: View Task History

```bash
# See the complete history of a task
frump history 1
```

Output shows all commits that affected task 1:
```
History for Task 1:

✓ Created by Alice Johnson on 2025-11-24 10:00
  Commit: a3f8b901
  Message: Add initial project tasks

• Modified by Alice Johnson on 2025-11-24 10:15
  Commit: c2e4d678
  Message: Update task 1 status to working

✗ Deleted by Alice Johnson on 2025-11-24 11:30
  Commit: 9f2b3c45
  Message: Complete task 1: Project structure set up
```

### Step 14: View Closed Tasks

```bash
# See all tasks you've completed
frump closed
```

Output:
```
Closed tasks:

Task 1 - Set up project structure

Total: 1 closed tasks
```

## Part 6: Advanced Features

### Step 15: Create a Template

Let's create a template for bug reports:

```bash
# Create bug template
frump template add bug "Fix {component} bug" \
  -t Bug \
  -b "Bug found in {component}.

Steps to reproduce:
{steps}

Expected behavior:
{expected}

Actual behavior:
{actual}"

# List your templates
frump template list
```

### Step 16: Use the Template

Now when you find a bug:

```bash
# Note: Template instantiation would require extending the add command
# For now, templates are stored and can be viewed
frump template show bug
```

### Step 17: Bulk Operations

Let's say you finish multiple tasks at once:

```bash
# First, mark them as done
frump set 2 Status done
frump set 3 Status done
frump set 4 Status done

# Close all done tasks at once
frump bulk close-by-status done

# Commit
git add frump.md
git commit -m "Close all completed tasks"
```

### Step 18: Export Your Tasks

Backup or share your tasks:

```bash
# Export to JSON
frump export -o backup.json

# Export to CSV for spreadsheet
frump export -f csv -o tasks.csv
```

## Part 7: Team Collaboration

### Step 19: Simulate a Merge Conflict

In real projects with multiple team members, you might get ID conflicts:

```bash
# Create a scenario:
# 1. Create a branch
git checkout -b feature/new-tasks

# 2. Add a task
frump add "Implement caching" -t Feature

# 3. Go back to main and add a different task
git checkout main
frump add "Add error handling" -t Task

# 4. Merge the branch
git merge feature/new-tasks
# If both tasks got ID 5, we have a conflict!
```

### Step 20: Resolve Conflicts

```bash
# Check for conflicts
frump check-conflicts

# If conflicts found, resolve them
frump resolve-conflicts --commit

# Verify everything is good
frump validate
frump list
```

## Part 8: Daily Workflow

### A Typical Day with Frump

**Morning:**
```bash
# See what you're working on
frump list -a "Your Name" -s working

# Review all open tasks
frump list -s open
```

**During Development:**
```bash
# Found a bug while coding
frump add -t Bug "Null pointer in user profile" \
  -b "Error occurs when user has no avatar" \
  -s "working"

# Start working on planned task
frump set 7 Status working

# Update task as you learn more
frump update 7 --body "Additional details discovered during implementation..."
```

**Code Review:**
```bash
# Teammate asks for a bug fix
frump add -t Bug "Memory leak in connection pool" \
  -a "Teammate Name"

# Update task based on review comments
frump set 10 Priority high
frump set 10 "Review Status" "changes requested"
```

**End of Day:**
```bash
# Complete finished tasks
frump close 8
frump close 9

# Commit your changes
git add frump.md
git commit -m "Update tasks: completed #8 and #9"

# See tomorrow's work
frump list -s working
```

## Best Practices

### 1. Commit Frequently
Every time you modify frump.md, commit it with a clear message:
```bash
git add frump.md
git commit -m "Add task 15: Implement notification system"
```

### 2. Use Descriptive Task Names
- Good: "Fix memory leak in WebSocket connection handler"
- Bad: "Fix bug"

### 3. Add Context in Task Body
```bash
frump add "Optimize database queries" \
  -b "Current queries are slow for large datasets.
Target: < 100ms for 1M records.
Focus on user search and report generation."
```

### 4. Keep Tasks Actionable
Each task should be something one person can complete:
- Good: "Add password reset email template"
- Bad: "Improve security" (too vague)

### 5. Use Properties Consistently
Decide on property names as a team:
- Status values: open, working, review, done, blocked
- Priority values: low, medium, high, critical

### 6. Regular Cleanup
```bash
# Weekly: close completed tasks
frump bulk close-by-status done
git add frump.md
git commit -m "Weekly cleanup: close completed tasks"

# Monthly: export for records
frump export -o "archive/tasks-$(date +%Y-%m).json"
```

## Next Steps

Now that you've completed the tutorial:

1. **Read the [USAGE.md](USAGE.md)** for complete command reference
2. **Review [README.md](README.md)** for concepts and philosophy
3. **Check [PLAN.md](PLAN.md)** if you want to contribute

## Common Questions

### Q: Can I have multiple frump.md files?
Yes! Use the `-f` flag:
```bash
frump --file docs/tasks.md list
frump -f backend/tasks.md show 5
```

### Q: What if I don't use git?
Frump works without git, but you'll lose:
- Task history (`frump history`)
- Closed task list (`frump closed`)
- Automatic ID conflict prevention

### Q: Can I customize the task types?
Yes! Use any type you want:
```bash
frump add -t Chore "Update dependencies"
frump add -t Documentation "Write API guide"
```

### Q: How do I recover a deleted task?
Check git history:
```bash
# See closed tasks
frump closed

# View when it was closed
frump history <task_id>

# Restore from git if needed
git log --all -- frump.md
git show <commit-hash>:frump.md > frump.md.old
# Copy the task back manually
```

### Q: Can I automate frump?
Yes! Frump is scriptable:
```bash
# Add tasks from a script
for feature in feature1 feature2 feature3; do
  frump add -t Feature "Implement $feature"
done

# Export for CI/CD
frump export -o tasks.json
# Process tasks.json in your pipeline
```

## Troubleshooting

### Tasks Not Showing Up
```bash
# Check file location
pwd
ls frump.md

# Validate file
frump validate
```

### Git History Not Working
```bash
# Make sure you're in a git repo
git status

# Make sure frump.md is committed
git log -- frump.md
```

### Property Validation Errors
Property keys must:
- Start with uppercase letter
- Be maximum 3 words

```bash
# Wrong:
frump set 1 priority high           # lowercase
frump set 1 "expected completion date" "2025-12-31"  # 3+ words

# Right:
frump set 1 Priority high
frump set 1 "Due Date" "2025-12-31"
```

## Congratulations!

You've completed the Frump tutorial. You now know how to:
- ✅ Set up a Frump project
- ✅ Create and manage tasks
- ✅ Use git integration
- ✅ Filter and search tasks
- ✅ Use advanced features
- ✅ Work effectively in a team

Happy task management! 🎉
