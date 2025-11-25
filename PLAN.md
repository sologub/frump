# Frump Implementation Plan

## Overview

This document outlines a comprehensive plan for implementing the full Frump system as described in README.md, including an improved Rust type system design and a roadmap for all features.

## 1. Improved Rust Type System Design

### 1.1 Core Domain Types

#### TaskId
```rust
/// Represents a unique task identifier
/// Task IDs are positive integers that must be unique across the entire history
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(u32);

impl TaskId {
    pub fn new(id: u32) -> Result<Self> {
        if id == 0 {
            return Err(anyhow!("Task ID must be positive"));
        }
        Ok(TaskId(id))
    }

    pub fn next(&self) -> TaskId {
        TaskId(self.0 + 1)
    }
}
```

**Rationale:** Strong typing prevents accidental misuse and makes the domain model clearer. The validation ensures IDs are always positive.

#### TaskType
```rust
/// Common task types with extensibility for custom types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    Task,
    Bug,
    Issue,
    Feature,
    Custom(String),
}

impl TaskType {
    pub fn parse(s: &str) -> Self {
        match s {
            "Task" => TaskType::Task,
            "Bug" => TaskType::Bug,
            "Issue" => TaskType::Issue,
            "Feature" => TaskType::Feature,
            _ => TaskType::Custom(s.to_string()),
        }
    }
}
```

**Rationale:** Provides standard types while allowing custom types. This maintains flexibility while providing compile-time guarantees for common cases.

#### Property
```rust
/// A task property with validated key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    key: PropertyKey,
    value: String,
}

/// A validated property key (capitalized, max 3 words)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyKey(String);

impl PropertyKey {
    pub fn new(key: &str) -> Result<Self> {
        // Must start with uppercase
        if !key.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            return Err(anyhow!("Property key must start with uppercase"));
        }

        // Max 3 words
        if key.split_whitespace().count() > 3 {
            return Err(anyhow!("Property key must have max 3 words"));
        }

        Ok(PropertyKey(key.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Predefined common properties
impl PropertyKey {
    pub fn status() -> Self {
        PropertyKey("Status".to_string())
    }

    pub fn assigned_to() -> Self {
        PropertyKey("Assigned To".to_string())
    }

    pub fn tags() -> Self {
        PropertyKey("Tags".to_string())
    }

    pub fn priority() -> Self {
        PropertyKey("Priority".to_string())
    }

    pub fn due_date() -> Self {
        PropertyKey("Due Date".to_string())
    }
}
```

**Rationale:** Validates property keys according to the format spec. Using a newtype pattern ensures validation happens at construction time.

#### Task (Enhanced)
```rust
/// Represents a task with all its metadata
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub task_type: TaskType,
    pub subject: String,
    pub body: String,
    pub properties: Vec<Property>,
}

impl Task {
    pub fn new(id: TaskId, task_type: TaskType, subject: String) -> Self {
        Task {
            id,
            task_type,
            subject,
            body: String::new(),
            properties: Vec::new(),
        }
    }

    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self
    }

    pub fn add_property(&mut self, key: PropertyKey, value: String) {
        self.properties.push(Property { key, value });
    }

    pub fn get_property(&self, key: &PropertyKey) -> Option<&str> {
        self.properties
            .iter()
            .find(|p| &p.key == key)
            .map(|p| p.value.as_str())
    }

    pub fn assignee(&self) -> Option<&str> {
        self.get_property(&PropertyKey::assigned_to())
    }

    pub fn status(&self) -> Option<&str> {
        self.get_property(&PropertyKey::status())
    }
}
```

**Rationale:** Builder pattern makes task construction ergonomic. Properties use Vec to preserve order (important for output). Convenience methods for common properties.

#### TeamMember (Enhanced)
```rust
/// Represents a team member
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamMember {
    pub name: String,
    pub email: Email,
    pub role: Option<String>,
}

/// Validated email address
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(email: &str) -> Result<Self> {
        // Basic email validation
        if !email.contains('@') {
            return Err(anyhow!("Invalid email format"));
        }
        Ok(Email(email.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TeamMember {
    pub fn new(name: String, email: Email) -> Self {
        TeamMember {
            name,
            email,
            role: None,
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.role = Some(role);
        self
    }
}
```

**Rationale:** Email validation prevents malformed data. Separate type makes the domain model more explicit.

#### FrumpDoc (Enhanced)
```rust
/// The complete frump.md document structure
#[derive(Debug)]
pub struct FrumpDoc {
    pub header: String,
    pub team: Team,
    pub tasks: TaskCollection,
}

/// Team with default assignee logic
#[derive(Debug, Clone)]
pub struct Team {
    members: Vec<TeamMember>,
}

impl Team {
    pub fn new(members: Vec<TeamMember>) -> Self {
        Team { members }
    }

    pub fn members(&self) -> &[TeamMember] {
        &self.members
    }

    /// Returns the default assignee (first team member)
    pub fn default_assignee(&self) -> Option<&TeamMember> {
        self.members.first()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&TeamMember> {
        self.members.iter().find(|m| m.name == name)
    }

    pub fn find_by_email(&self, email: &str) -> Option<&TeamMember> {
        self.members.iter().find(|m| m.email.as_str() == email)
    }
}

/// Collection of tasks with useful operations
#[derive(Debug)]
pub struct TaskCollection {
    tasks: Vec<Task>,
}

impl TaskCollection {
    pub fn new(tasks: Vec<Task>) -> Self {
        TaskCollection { tasks }
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn tasks_mut(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }

    pub fn find_by_id(&self, id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    pub fn max_id(&self) -> Option<TaskId> {
        self.tasks.iter().map(|t| t.id).max()
    }

    pub fn next_id(&self) -> TaskId {
        self.max_id()
            .map(|id| id.next())
            .unwrap_or(TaskId::new(1).unwrap())
    }

    pub fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn remove(&mut self, id: TaskId) -> Option<Task> {
        let pos = self.tasks.iter().position(|t| t.id == id)?;
        Some(self.tasks.remove(pos))
    }

    pub fn filter_by_assignee(&self, assignee: &str) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.assignee().map(|a| a == assignee).unwrap_or(false))
            .collect()
    }

    pub fn filter_by_status(&self, status: &str) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.status().map(|s| s == status).unwrap_or(false))
            .collect()
    }

    pub fn filter_by_type(&self, task_type: &TaskType) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| &t.task_type == task_type)
            .collect()
    }
}

impl FrumpDoc {
    pub fn new(header: String, team: Team, tasks: TaskCollection) -> Self {
        FrumpDoc { header, team, tasks }
    }

    /// Apply default assignee to tasks that don't have one
    pub fn apply_default_assignees(&mut self) {
        if let Some(default) = self.team.default_assignee() {
            for task in self.tasks.tasks_mut() {
                if task.assignee().is_none() {
                    task.add_property(
                        PropertyKey::assigned_to(),
                        default.name.clone(),
                    );
                }
            }
        }
    }
}
```

**Rationale:** Encapsulates team and task logic. The Team type handles default assignee logic. TaskCollection provides rich querying and manipulation capabilities.

### 1.2 Git Integration Types

```rust
/// Git repository wrapper for Frump operations
pub struct FrumpRepo {
    repo_path: PathBuf,
    frump_file: PathBuf,
}

/// A historical snapshot of a task from git history
#[derive(Debug, Clone)]
pub struct TaskHistory {
    pub task_id: TaskId,
    pub commits: Vec<TaskCommit>,
}

/// A commit that affected a task
#[derive(Debug, Clone)]
pub struct TaskCommit {
    pub hash: String,
    pub author: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

impl FrumpRepo {
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        // Validate it's a git repo
        Ok(FrumpRepo {
            repo_path,
            frump_file: PathBuf::from("frump.md"),
        })
    }

    /// Find the maximum task ID ever used in git history
    pub fn max_historical_id(&self) -> Result<Option<TaskId>> {
        // Parse all historical versions of frump.md
        // Return the maximum ID ever seen
        todo!("Implement git history parsing")
    }

    /// Get the history of a specific task
    pub fn task_history(&self, id: TaskId) -> Result<TaskHistory> {
        todo!("Implement task history from git log")
    }

    /// List all tasks that have been deleted (in history but not current)
    pub fn deleted_tasks(&self) -> Result<Vec<TaskId>> {
        todo!("Compare current with historical tasks")
    }
}
```

**Rationale:** Separates git operations into a dedicated type. Provides type-safe representation of historical data.

### 1.3 Module Organization

```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library root
├── domain/
│   ├── mod.rs
│   ├── task.rs            # Task, TaskId, TaskType
│   ├── property.rs        # Property, PropertyKey
│   ├── team.rs            # Team, TeamMember, Email
│   └── document.rs        # FrumpDoc, TaskCollection
├── parser/
│   ├── mod.rs
│   └── markdown.rs        # Markdown parsing/serialization
├── git/
│   ├── mod.rs
│   └── repository.rs      # FrumpRepo, git operations
├── commands/
│   ├── mod.rs
│   ├── list.rs            # List command
│   ├── show.rs            # Show command
│   ├── add.rs             # Add command
│   ├── close.rs           # Close command
│   ├── history.rs         # History command
│   ├── assign.rs          # Assign command
│   └── set.rs             # Set property command
└── cli.rs                 # CLI argument parsing
```

**Rationale:** Clear separation of concerns. Domain types are independent of parsing and git operations. Commands are isolated for testability.

## 2. Implementation Roadmap

### Phase 1: Core Type System Refactoring ✓ (Current POC)

**Status:** Partially complete
- [x] Basic Task parsing
- [x] Basic Team parsing
- [x] Basic CLI with list/show/add
- [ ] Property validation
- [ ] Strong typing for domain types
- [ ] TaskCollection with rich operations

### Phase 2: Enhanced Domain Model

**Goal:** Implement the improved type system from Section 1.1

**Tasks:**
1. Create `domain/` module structure
2. Implement TaskId with validation
3. Implement TaskType enum
4. Implement Property and PropertyKey with validation
5. Implement enhanced Task with builder pattern
6. Implement Email validation for TeamMember
7. Implement Team with default assignee logic
8. Implement TaskCollection with filtering
9. Update parser to use new types
10. Update CLI commands to use new types
11. Add comprehensive unit tests for domain types

**Estimated effort:** 2-3 days

### Phase 3: Git Integration

**Goal:** Implement git history inspection and task history features

**Tasks:**
1. Add `git2` dependency to Cargo.toml
2. Create `git/` module structure
3. Implement FrumpRepo initialization and validation
4. Implement `max_historical_id()` to scan git history
   - Use `git log --all -- frump.md` to get all commits
   - Parse each version of frump.md from each commit
   - Track maximum ID seen
5. Implement `task_history()` to get changes to a specific task
   - Parse git log for changes mentioning task ID
   - Detect creation, modification, deletion
6. Update `add` command to use max_historical_id()
7. Add `history` command to show task history
8. Add `closed` command to list deleted tasks

**Estimated effort:** 3-4 days

### Phase 4: Extended Commands

**Goal:** Complete the command set for full task management

**Commands to implement:**

#### 4.1 `frump close <id>`
Close a task by removing it from frump.md

```bash
frump close 10
# Removes task 10 from frump.md
# User should then commit with descriptive message
```

#### 4.2 `frump assign <id> <name>`
Assign a task to a team member

```bash
frump assign 10 "John Doe"
# Adds/updates "Assigned To: John Doe" property
```

#### 4.3 `frump set <id> <property> <value>`
Set a property on a task

```bash
frump set 10 Status "in progress"
frump set 10 Priority high
```

#### 4.4 `frump update <id>`
Update task subject or body interactively

```bash
frump update 10
# Opens editor with current task content
```

#### 4.5 `frump filter --status <status> --assignee <name> --type <type>`
Filter tasks by various criteria

```bash
frump filter --status working --assignee "Ruslan"
frump filter --type Bug
```

#### 4.6 `frump history <id>`
Show the complete history of a task from git

```bash
frump history 10
# Shows all commits that modified task 10
```

#### 4.7 `frump closed`
List all closed (deleted) tasks

```bash
frump closed
# Lists task IDs and subjects of all tasks removed from frump.md
```

**Estimated effort:** 4-5 days

### Phase 5: Validation and Error Handling

**Goal:** Robust validation and helpful error messages

**Tasks:**
1. Implement comprehensive property key validation
2. Validate task ID uniqueness
3. Validate team member email formats
4. Detect and warn about potential merge conflicts
5. Add helpful error messages for common mistakes
6. Validate frump.md format before parsing
7. Add `frump validate` command to check file integrity

**Estimated effort:** 2 days

### Phase 6: Advanced Features

**Goal:** Advanced functionality for power users

**Features:**

#### 6.1 Task Templates
```bash
frump add-template bug "Bug report"
frump add --template bug "login fails"
```

#### 6.2 Bulk Operations
```bash
frump close-by-status done
frump assign-by-type Bug "QA Team"
```

#### 6.3 Search
```bash
frump search "authentication"
# Full-text search across task subjects and bodies
```

#### 6.4 Statistics
```bash
frump stats
# Shows task count by type, status, assignee
```

#### 6.5 Export
```bash
frump export --format json > tasks.json
frump export --format csv > tasks.csv
```

#### 6.6 Import
```bash
frump import tasks.json
# Import tasks from JSON (e.g., from another tool)
```

**Estimated effort:** 5-6 days

### Phase 7: Merge Conflict Assistance

**Goal:** Help users resolve task ID conflicts during git merges

**Tasks:**
1. Implement `frump check-conflicts` command
2. Detect duplicate task IDs in frump.md
3. Implement `frump resolve-conflicts` command
4. Automatically renumber conflicting tasks
5. Generate commit with conflict resolution

**Estimated effort:** 2-3 days

### Phase 8: Testing and Documentation

**Goal:** Comprehensive test coverage and documentation

**Tasks:**
1. Unit tests for all domain types (target 90%+ coverage)
2. Integration tests for all commands
3. Test with various edge cases:
   - Empty frump.md
   - Missing sections
   - Malformed markdown
   - Large files with many tasks
4. Write comprehensive README with examples
5. Write USAGE.md with all commands documented
6. Add docstrings to all public APIs
7. Create tutorial for new users

**Estimated effort:** 4-5 days

## 3. Implementation Priorities

### Must Have (MVP)
1. Enhanced domain model with validation (Phase 2)
2. Git history integration for max ID (Phase 3, partial)
3. Core commands: list, show, add, close, assign, set (Phase 4, subset)
4. Basic validation (Phase 5, subset)

### Should Have
5. Full git history features (Phase 3, complete)
6. All extended commands (Phase 4, complete)
7. Comprehensive validation (Phase 5, complete)
8. Search and filter capabilities (Phase 6, subset)

### Nice to Have
9. Templates and bulk operations (Phase 6)
10. Export/import (Phase 6)
11. Merge conflict assistance (Phase 7)

## 4. Design Principles

### 4.1 Type Safety
Use Rust's type system to enforce invariants at compile time:
- TaskId must be positive
- PropertyKey must be validated
- Email must be well-formed

### 4.2 Fail Fast
Validate data as early as possible. Construction time is better than usage time.

### 4.3 Explicit Over Implicit
Make the domain model explicit. Use newtypes liberally.

### 4.4 Composability
Small, focused types that compose well:
- Task doesn't know about git
- Parser doesn't know about git
- Commands orchestrate domain types

### 4.5 Testability
Every component should be independently testable:
- Pure functions where possible
- Dependency injection for effects (filesystem, git)
- Clear separation of parsing, domain logic, and output

### 4.6 User Experience
- Helpful error messages with suggestions
- Sane defaults (default assignee, Task type)
- Interactive modes for complex operations
- Colorized output for better readability

## 5. Testing Strategy

### 5.1 Unit Tests
- Test each domain type independently
- Test validation rules
- Test edge cases (empty strings, max values, etc.)

### 5.2 Integration Tests
- Test each command end-to-end
- Use temporary directories and git repos
- Test with various frump.md formats

### 5.3 Property-Based Tests
Use `proptest` or `quickcheck` for:
- Parser round-trip (parse → serialize → parse should equal original)
- Property validation
- Task ID generation

### 5.4 Snapshot Tests
Use `insta` for:
- Command output
- Markdown serialization
- Error messages

## 6. Future Enhancements

### 6.1 Web Interface
- Local web UI to visualize tasks
- Kanban board view
- Timeline view from git history

### 6.2 GitHub Integration
- Sync with GitHub Issues
- Create PR from task
- Link tasks to commits

### 6.3 Configuration
- `.frumprc` file for defaults
- Per-repo configuration
- Custom property types

### 6.4 Hooks
- Pre-add hooks for validation
- Post-close hooks for notifications
- Custom workflow automations

### 6.5 Multi-file Support
- Split large projects across multiple frump files
- Per-component task lists
- Aggregated views

## 7. Open Questions

1. **Property Types:** Should properties have typed values (dates, numbers) or stay as strings?
   - *Recommendation:* Start with strings for simplicity, add typed properties later if needed

2. **Task References:** Should tasks be able to reference other tasks?
   - *Recommendation:* Add in Phase 6 as "Related Tasks" property

3. **Subtasks:** Should tasks support hierarchical subtasks?
   - *Recommendation:* Not in MVP. Consider for later phases.

4. **Attachments:** Should tasks support file attachments?
   - *Recommendation:* No. Files should be in git. Link to them in task body.

5. **Task Numbering on Conflict:** What strategy for merge conflicts?
   - *Recommendation:* Renumber conflicting tasks to next available IDs. Phase 7 feature.

## 8. Success Metrics

- Parse and round-trip any valid frump.md file without data loss
- Handle git repos with 1000+ tasks efficiently
- Command execution < 100ms for most operations (excluding git operations)
- 90%+ test coverage for domain logic
- Zero panics in normal operation (all errors handled gracefully)

## Conclusion

This plan provides a solid foundation for building a complete, production-ready Frump implementation. The type system improvements will make the code more maintainable and prevent entire classes of bugs. The phased approach allows for incremental development while always having a working system.
