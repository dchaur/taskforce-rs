# 📋 Advanced Task Management System (`todo-cli`) 🦀

> **A high-performance, production-grade terminal application built from scratch with Rust utilizing strict Test-Driven Development (TDD) principles.**

[![Rust Performance](https://img.shields.io/badge/Rust-E05539?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Testing Suite](https://img.shields.io/badge/Tests-85%20Passing-emerald?style=flat-square&logo=github-actions&logoColor=white)](https://github.com)
[![Architecture](https://img.shields.io/badge/Architecture-Modular%20%26%20Event--Driven-blue?style=flat-square)](https://github.com)

After mastering core memory safety, ownership, and concurrency patterns in Rust, I engineered this system to solidify production-level concepts. Moving beyond standard tutorial implementations, this project focuses on robust architecture, data integrity, and cross-interface modularity.

---

## 🛠️ Software Engineering & Architecture

This system was engineered with a strict separation of concerns, ensuring that the underlying domain and business logic remain completely decoupled from the presentation layers.

* **Strict TDD Approach:** Built using the 🔴 Red, 🟢 Green, 🔃 Refactor cycle. Every core feature, edge case, and edge-triggered error path was covered by a test before implementation.
* **Comprehensive Test Suite:** Packed with **85 tests**, featuring fine-grained unit testing for core data structures, deterministic formatter validation, and **robust End-to-End (E2E) CLI integration tests** simulating real binary executions.
* **Decoupled Modular Architecture:** Structured cleanly into specialized crates/modules:
    * `domain`: Immutable models for tasks, tags, priority matrices, and chronological state.
    * `todo_list`: Core business logic engines and transactional mutations.
    * `commands`: High-level orchestrators processing standard input parameters.
    * `utils`: Thread-safe persistence systems (JSON parsing engines and automated CSV exporters).

---

## ✨ Three Interface Modes

The system abstracts business operations to drive **three completely distinct presentation interfaces** seamlessly:

1.  **Direct CLI Mode:** Fast, pipeline-friendly, and fully scriptable command execution engine.
2.  **Interactive Menu Mode:** A user-friendly, prompt-driven terminal interface powered by `dialoguer` for fast data entry.
3.  **Full-Screen TUI Mode:** A rich, immersive terminal UI leveraging event-driven rendering with real-time navigation, Vim-style keybindings, task toggling, and asynchronous color-coded state management.

### Key Engine Features:
* **Persistent Transaction Layer:** Immediate thread-safe serialization/deserialization to standard JSON file targets surviving session teardowns.
* **Transactional Undo System:** In-memory command pattern implementation capable of rolling back mutations (additions, modifications, completions, or deletions).
* **Advanced Evaluation Engine:** Dynamic runtime tags analytics (`#work`, `#urgent`), multi-tag matching (OR evaluation structures), and automatic boundary overdue validation checks.

---

## 🔧 Usage

```bash
# Show version
cargo run -- --version

# List all tasks
cargo run -- list

# List only todo tasks
cargo run -- list --status todo

# List only completed tasks
cargo run -- list --status done

# Add a new task (defaults to medium priority)
cargo run -- add "My new task"

# Add a task with priority
cargo run -- add "Deploy to production" --priority high
cargo run -- add "Fix typo" --priority low

# Add a task with a due date
cargo run -- add "Submit report" --due 2026-05-15

# Add a task with both priority and due date
cargo run -- add "Review code" --priority high --due 2026-05-10

# Add a task with tags
cargo run -- add "Deploy API #work #urgent"

# List tasks filtered by tag
cargo run -- list --tag work

# List tasks with multiple tags (OR logic)
cargo run -- list --tag work,personal

# List untagged tasks
cargo run -- list --tag none

# Search for tasks by keyword (case-insensitive)
cargo run -- find "rust"

# Count tasks
cargo run -- count

# Edit/rename a task
cargo run -- edit "My new task" "My updated task"

# Mark a task as completed
cargo run -- mark-done "My updated task"

# Mark a task as to-do (undo completion)
cargo run -- mark-todo "My updated task"

# Remove a specific task
cargo run -- remove "My updated task"

# Remove all completed tasks
cargo run -- clear-done

# Remove all todo tasks
cargo run -- clear-todo

# Remove all tasks
cargo run -- clear-all

# Export tasks to CSV file
cargo run -- export tasks.csv

# Export tasks to stdout (display CSV)
cargo run -- export

# Undo the last operation
cargo run -- undo

# Run interactive mode (menu-driven interface)
cargo run -- interactive

# Run TUI mode (full-screen terminal UI)
cargo run -- tui
# TUI controls: ↑↓ or j/k to navigate, 'd' toggle done, 'x' delete, 'q' quit
```

## 🧪 Testing

Run all tests with:

```bash
cargo test
```

This executes 85 tests including unit tests, formatter tests, and end-to-end integration tests.
