# 🗺️ Feature Roadmap

> _Learning Rust through incremental feature development with TDD_

This roadmap tracks the journey from basic CLI to a full-featured task management system. Each feature is designed to teach new Rust concepts while following Test-Driven Development.

---

## ✅ Completed Features

- [x] **Basic CRUD operations** - Add, list, mark, remove tasks
- [x] **JSON persistence** - Save/load with serde
- [x] **Status filtering** - List by todo/done status
- [x] **Batch operations** - Clear all/done/todo tasks
- [x] **Edit/rename** - Rename tasks while preserving status
- [x] **Architecture refactor** - Split into lib.rs + main.rs
- [x] **Comprehensive testing** - 59 tests (unit + integration + e2e)
- [x] **Count command** - Aggregate counts with proper pluralization
- [x] **Priority levels** - High/Medium/Low with visual indicators (⬆️ ➡️ ⬇️)
- [x] **Due dates** - Date validation, overdue detection with visual warnings (🔴 OVERDUE)
- [x] **Search/Find** - Case-insensitive keyword search across tasks
- [x] **Tags/Categories** - HashSet-based tagging with # syntax and filtering
- [x] **CSV Export** - Export tasks to CSV with proper escaping and stdout support
- [x] **Undo command** - Undo last add/edit/mark/delete operation
- [x] **Modular refactoring** - Clean architecture with separated modules (utils, domain, todo_list, commands)

---

## 🎯 Phase 1: Easy Wins (Core Extensions)

### 1.1 Count Command

**Status**: ✅ Complete  
**Estimated Time**: 30-40 minutes

**What you'll learn**:

- Aggregation and counting
- Tuple returns for multiple values
- String formatting with counts

**TDD Steps**:

1. 🔴 Write test: `count()` returns (total, todo_count, done_count)
2. 🟢 Implement method on TodoList
3. 🔴 Write test: `handle_command` with "count"
4. 🟢 Implement in handle_command
5. 🔴 Write e2e test: verify output format
6. 🟢 Implement formatter in main.rs
7. 🔃 Refactor for clarity

**Example usage**:

```bash
cargo run -- count
# Output:
# Total: 10 tasks
# To Do: 6 tasks
# Done: 4 tasks
```

---

### 1.2 Priority Levels

**Status**: ✅ Complete  
**Estimated Time**: 1-2 hours
**Completed**: May 6, 2026

**What you'll learn**:

- Extending enums with more variants
- Default values (derive Default or custom implementation)
- Optional command arguments
- Ordering and sorting by priority

**TDD Steps**:

1. 🔴 Create Priority enum (Low, Medium, High)
2. 🔴 Update TodoItem struct (new struct with name, status, priority)
3. 🔴 Refactor HashMap to use TodoItem as value
4. 🟢 Migrate existing tests
5. 🔴 Add --priority flag to add command
6. 🟢 Implement priority assignment (default to Medium)
7. 🔴 Update list to show priorities
8. 🟢 Format output with priority indicators (e.g., ⬆️ High, ➡️ Medium, ⬇️ Low)
9. 🔃 Refactor and add edge cases

**Example usage**:

```bash
cargo run -- add "Deploy to prod" --priority high
cargo run -- list
# Output:
# To Do:
#  * ⬆️ Deploy to prod
#  * ➡️ Write tests
```

---

### 1.3 Due Dates

**Status**: ✅ Complete  
**Estimated Time**: 2-3 hours
**Completed**: May 7-8, 2026

**What you'll learn**:

- Working with dates in Rust (chrono crate)
- Parsing date strings
- Comparing dates and showing "overdue" status
- Option<T> for optional fields
- Result error handling for invalid dates

**TDD Steps**:

1. ✅ Add chrono dependency to Cargo.toml
2. ✅ Add due_date: Option<NaiveDate> to TodoItem
3. ✅ Write test: parse due date from string
4. ✅ Implement date parsing with error handling
5. ✅ Add --due flag to add/edit commands
6. ✅ Implement due date assignment
7. ✅ Write test: identify overdue tasks
8. ✅ Implement overdue detection (compare with today)
9. ✅ Update formatter to show due dates
10. ✅ Show due dates with overdue warnings (🔴 OVERDUE)
11. ✅ Refactor and handle edge cases (including future date validation)

**Example usage**:

```bash
cargo run -- add "Submit report" --due 2026-05-10
cargo run -- list
# Output:
# To Do:
#  * Submit report (due: 2026-05-10)
#  * 🔴 OVERDUE: Pay bills (due: 2026-05-01)
```

---

## 🚀 Phase 2: Intermediate Features (New Concepts)

### 2.1 Search/Find Command

**Status**: ✅ Complete  
**Estimated Time**: 45 minutes - 1 hour
**Completed**: May 12, 2026

**What you'll learn**:

- String matching and contains()
- Case-insensitive search (to_lowercase())
- Filter iterators with closures
- Collecting results into Vec

**TDD Steps**:

1. ✅ Write test: find tasks containing keyword
2. ✅ Implement find() method returning Vec<ListInfo>
3. ✅ Write test: case-insensitive search
4. ✅ Convert to lowercase for comparison
5. ✅ Add find command to handle_command
6. ✅ Implement command handling
7. ✅ Write e2e test (including empty results case)
8. ✅ Implement output formatter (reused existing format_section!)
9. ✅ Refactor

**Example usage**:

```bash
cargo run -- find "rust"
# Output:
# Found 2 tasks:
#
# To Do
#  * ⬆️  Learn Rust basics (due: 2026-05-15)
#
# Done
#  * ➡️  Practice Rust iterators
```

---

### 2.2 Tags/Categories

**Status**: ✅ Complete  
**Estimated Time**: 2-3 hours
**Completed**: May 9-11, 2026

**What you'll learn**:

- HashSet for unique collections
- Parsing tags from strings (#work, #personal)
- Multiple filters (status + tag)
- More complex data structures

**TDD Steps**:

1. ✅ Add tags: HashSet<String> to TodoItem
2. ✅ Write test: parse tags from task name
3. ✅ Implement tag extraction (find words starting with #)
4. ✅ Write test: filter by tag
5. ✅ Implement tag filtering
6. ✅ Add --tag flag to list command
7. ✅ Implement combined filters (status + tag)
8. ✅ Update formatter to show tags
9. ✅ Display tags in output
10. ✅ Refactor (including OR logic with comma-separated tags and 'none' filter)

**Example usage**:

```bash
cargo run -- add "Deploy API #work #urgent"
cargo run -- list --tag work
# Output:
# To Do:
#  * Deploy API #work #urgent
#  * Review PRs #work
```

---

### 2.3 Export to CSV

**Status**: ✅ Complete  
**Estimated Time**: 1-2 hours
**Completed**: May 11, 2026

**What you'll learn**:

- File I/O with std::fs
- CSV formatting
- Writing to files with error handling
- Path handling

**TDD Steps**:

1. ✅ Write test: export generates valid CSV content
2. ✅ Implement CSV formatting (task,status,priority,due_date,tags)
3. ✅ Write test: export writes to file
4. ✅ Implement file writing with error handling
5. ✅ Add export command
6. ✅ Implement command handling with file path argument
7. ✅ Write e2e test: verify file creation
8. ✅ Complete implementation (including stdout export and quote escaping)
9. ✅ Refactor

**Example usage**:

```bash
cargo run -- export tasks.csv
# Successfully exported 10 tasks to tasks.csv
```

---

### 2.4 Undo Last Action

**Status**: ✅ Complete  
**Estimated Time**: 2-3 hours
**Completed**: May 12, 2026

**What you'll learn**:

- State management and history
- Vec for storing command history
- Cloning data structures
- More complex Result handling

**TDD Steps**:

1. ✅ Create UndoAction enum to track operation types
2. ✅ Write test: save action state before mutation
3. ✅ Implement snapshot with TodoItem cloning
4. ✅ Write test: undo restores previous state
5. ✅ Implement undo() for add/edit/mark/delete
6. ✅ Add undo command
7. ✅ Integrate with handle_command
8. ✅ Write e2e test: add → undo → verify
9. ✅ Complete implementation (single-level undo)
10. ✅ Refactor and add error handling

**Example usage**:

```bash
cargo run -- remove "Important task"
cargo run -- undo
# Undid last action. Task restored!
```

---

## 🏆 Phase 3: Advanced Features (Reward Zone!)

### 3.1 Interactive Mode

**Status**: ✅ Complete  
**Estimated Time**: 2-4 hours
**Started**: May 14, 2026
**Completed**: May 15, 2026

**What you'll learn**:

- Using external crates (dialoguer)
- Interactive prompts and menus
- Loop-based CLI flows
- Better UX design

**Add to Cargo.toml**:

```toml
dialoguer = "0.11"
```

**Features to implement**:

- Menu-driven interface
- Arrow key navigation
- Interactive prompts for task details
- Task selection from lists
- All core operations in friendly UI

**Example**:

```bash
cargo run -- interactive
? What would you like to do?
❯ Add a new task
  List tasks
  Mark task as done
  Remove task
  Exit
```

---

### 3.2 Color Output

**Status**: ✅ Complete  
**Estimated Time**: 1-2 hours
**Completed**: May 15, 2026

**What you'll learn**:

- Using colored crate for terminal colors
- Making CLI output pretty
- Platform-specific handling (Windows vs Unix)

**Add to Cargo.toml**:

```toml
colored = "2.0"
```

**Features to implement**:

- Green for done tasks ✓
- Red for overdue tasks
- Yellow for warnings
- Bold for headers
- Dim for completed tasks

---

### 3.3 TUI (Terminal UI)

**Status**: ✅ Complete  
**Estimated Time**: 6-10 hours (BIG PROJECT!)
**Completed**: May 18-19, 2026

**What you'll learn**:

- Advanced terminal manipulation
- Event-driven architecture
- State management in TUI
- Using ratatui crate
- Real-time updates

**Add to Cargo.toml**:

```toml
ratatui = "0.26"
crossterm = "0.27"
```

**Features to implement**:

- Full-screen terminal interface
- Multiple panels (task list, details, help)
- Keyboard shortcuts (j/k for navigation, enter to edit, etc.)
- Real-time editing
- Beautiful ASCII UI

**Example vision**:

```
┌─ Todo CLI ─────────────────────────────────────────────┐
│ ⬆️  Deploy to prod                              [URGENT]│
│ ➡️  Write documentation                                │
│ ✓  Fix bug in auth                                     │
├────────────────────────────────────────────────────────┤
│ Selected: Deploy to prod                               │
│ Priority: High | Due: 2026-05-10 | Tags: #work #deploy│
├────────────────────────────────────────────────────────┤
│ [a]dd [e]dit [d]one [x]delete [f]ilter [q]uit        │
└────────────────────────────────────────────────────────┘
```

---

## 📊 Progress Tracker

**Overall Progress**: 20/20 features (100%) 🎉

- ✅ Phase 0 (Foundation): 10/10 (100%)
- ✅ Phase 1 (Easy Wins): 3/3 (100%) - Count ✅, Priority ✅, Due Dates ✅
- ✅ Phase 2 (Intermediate): 4/4 (100%) - Search/Find ✅, Tags ✅, CSV Export ✅, Undo ✅
- ✅ Phase 3 (Advanced): 3/3 (100%) - Interactive Mode ✅, Color Output ✅, TUI ✅

---

## � PROJECT COMPLETE!

**🏆 ALL 20 FEATURES IMPLEMENTED! 🏆**

**Final Achievement**: 🎮 TUI Complete! Full-screen terminal interface with navigation, toggle, delete with confirmation, and color-coded priorities! (May 18-19, 2026)

**Journey Highlights**:

- 🎊 Interactive Mode Complete! (May 15, 2026)
- 🎨 Color Output Complete! (May 15, 2026)
- 🎉 Phase 2 Complete! All 4 intermediate features! (May 14, 2026)
- 🏆 Major Refactoring! Clean modular architecture with 85 tests passing!
- 🦀 **First Rust project after "Learn to Code with Rust" course - CRUSHED IT!**

---

## 💡 Learning Notes

As you build each feature, consider documenting:

- Rust concepts learned
- Challenges faced
- Solutions discovered
- Patterns that clicked

This roadmap is your guide, but feel free to adjust the order or add features that interest you. The journey is yours! 🦀
