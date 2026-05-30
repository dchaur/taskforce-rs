use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration, Local};

fn unique_temp_dir() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock is before UNIX_EPOCH")
        .as_nanos();

    let dir =
        std::env::temp_dir().join(format!("todo_cli_it_{}_{}", std::process::id(), timestamp));
    fs::create_dir_all(&dir).expect("Failed to create temporary test directory");
    dir
}

fn run_cli(args: &[&str], dir: &PathBuf) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_todo-cli");
    Command::new(bin)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("Failed to execute todo-cli binary")
}

fn run_cli_stdout(args: &[&str], dir: &PathBuf, context: &str) -> String {
    let output = run_cli(args, dir);
    assert!(output.status.success(), "{context} should succeed");
    String::from_utf8(output.stdout).expect("CLI stdout should be valid UTF-8")
}

fn assert_contains_task_in_section(list_output: &str, section_header: &str, task_name: &str) {
    assert!(list_output.contains(section_header));
    assert!(list_output.contains(task_name));
}

#[test]
fn cli_flow_add_mark_done_and_list() {
    let dir = unique_temp_dir();
    let task_name = "Task A";

    let add_stdout = run_cli_stdout(&["add", task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let list_todo_stdout = run_cli_stdout(&["list"], &dir, "list after add");
    assert_contains_task_in_section(&list_todo_stdout, "# To Do", task_name);

    let mark_done_stdout = run_cli_stdout(&["mark-done", task_name], &dir, "mark-done");
    assert!(mark_done_stdout.contains("Successfully updated!"));

    let list_done_stdout = run_cli_stdout(&["list"], &dir, "list after mark-done");
    assert_contains_task_in_section(&list_done_stdout, "# Done", task_name);

    fs::remove_dir_all(&dir).expect("Failed to clean temporary test directory");
}

#[test]
fn cli_flow_add_mark_done_mark_todo_and_list() {
    let dir = unique_temp_dir();
    let task_name = "Learn Rust";

    let add_stdout = run_cli_stdout(&["add", task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let list_todo_stdout = run_cli_stdout(&["list"], &dir, "list after add");
    assert_contains_task_in_section(&list_todo_stdout, "# To Do", task_name);

    let mark_done_stdout = run_cli_stdout(&["mark-done", task_name], &dir, "mark-done");
    assert!(mark_done_stdout.contains("Successfully updated!"));

    let list_done_stdout = run_cli_stdout(&["list"], &dir, "list after mark-done");
    assert_contains_task_in_section(&list_done_stdout, "# Done", task_name);

    let mark_todo_stdout = run_cli_stdout(&["mark-todo", task_name], &dir, "mark-todo");
    assert!(mark_todo_stdout.contains("Successfully updated!"));

    let list_after_mark_todo_stdout = run_cli_stdout(&["list"], &dir, "list after mark-todo");
    assert_contains_task_in_section(&list_after_mark_todo_stdout, "# To Do", task_name);

    fs::remove_dir_all(&dir).expect("Failed to clean temporary test directory");
}

#[test]
fn cli_flow_add_edit_and_list() {
    let dir = unique_temp_dir();
    let old_task = "Learn Cobol";
    let new_task = "Learn Rust";

    let add_stdout = run_cli_stdout(&["add", old_task], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let list_todo_stdout = run_cli_stdout(
        &["list", "--status", "todo"],
        &dir,
        "list only todo after add",
    );
    assert_contains_task_in_section(&list_todo_stdout, "# To Do", old_task);
    assert!(!list_todo_stdout.contains("# Done")); // Verify no Done section when filtering by todo

    let edit_stdout = run_cli_stdout(&["edit", old_task, new_task], &dir, "edit");
    assert!(edit_stdout.contains("Successfully updated!"));

    let list_edited_stdout = run_cli_stdout(&["list", "--status", "todo"], &dir, "list after edit");

    assert_contains_task_in_section(&list_edited_stdout, "# To Do", new_task);
    assert!(!list_edited_stdout.contains("# Done"));
    assert!(!list_edited_stdout.contains(old_task));

    fs::remove_dir_all(&dir).expect("Failed to clean temporary test directory");
}

#[test]
fn cli_flow_count_items() {
    let dir = unique_temp_dir();
    let first_task_name = "Learn Cobol";
    let second_task_name = "Learn Assembler";
    let third_task_name = "Learn Rust";

    let add_stdout = run_cli_stdout(&["add", first_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", second_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", third_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let mark_done_stdout = run_cli_stdout(&["mark-done", second_task_name], &dir, "mark-done");
    assert!(mark_done_stdout.contains("Successfully updated!"));

    let count_list_stdout = run_cli_stdout(&["count"], &dir, "count items after mark-todo");

    assert!(count_list_stdout.contains("# Total: 3 tasks"));
    assert!(count_list_stdout.contains("# To Do: 2 tasks"));
    assert!(count_list_stdout.contains("# Done: 1 task"));

    fs::remove_dir_all(&dir).expect("Failed to clean temporary test directory");
}

#[test]
fn cli_flow_add_and_list_overdue_tasks() {
    let temp_dir = unique_temp_dir();
    let json_path = temp_dir.join("todos.json");
    let task_name = "Learn carpentry";
    let yesterday = Local::now().date_naive() - Duration::days(1);

    run_cli_stdout(&["add", task_name], &temp_dir, "add task");

    let json_content = format!(
        r#"{{"items":{{"{0}":{{"name":"{0}","status":"Todo","priority":"Medium","due_date":"{1}","tags":[]}}}}}}"#,
        task_name, yesterday
    );
    std::fs::write(&json_path, json_content).unwrap();

    let list_output = run_cli_stdout(&["list"], &temp_dir, "list tasks");
    assert!(list_output.contains("🔴 OVERDUE"));
}

#[test]
fn cli_flow_search_and_find_item() {
    let dir = unique_temp_dir();
    let first_task_name = "Learn Cobol";
    let second_task_name = "Practice Assembler";
    let third_task_name = "Learn Rust";

    let add_stdout = run_cli_stdout(&["add", first_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", second_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", third_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let mark_done_stdout = run_cli_stdout(&["mark-done", second_task_name], &dir, "mark-done");
    assert!(mark_done_stdout.contains("Successfully updated!"));

    let find_stdout = run_cli_stdout(&["find", "Learn"], &dir, "filter items by a keyword");

    assert!(find_stdout.contains("Learn Cobol"));
    assert!(find_stdout.contains("Learn Rust"));

    assert!(!find_stdout.contains("Practice Assembler"));

    assert!(find_stdout.contains("# To Do"));

    fs::remove_dir_all(&dir).expect("Failed to clean temporary test directory");
}

#[test]
fn cli_flow_search_no_result() {
    let dir = unique_temp_dir();
    let first_task_name = "Learn Cobol";
    let second_task_name = "Practice Assembler";
    let third_task_name = "Learn Rust";

    let add_stdout = run_cli_stdout(&["add", first_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", second_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", third_task_name], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let mark_done_stdout = run_cli_stdout(&["mark-done", second_task_name], &dir, "mark-done");
    assert!(mark_done_stdout.contains("Successfully updated!"));

    let find_stdout = run_cli_stdout(&["find", "Play"], &dir, "filter items by a keyword");
    assert!(find_stdout.contains("No Tasks found."));

    fs::remove_dir_all(&dir).expect("Failed to clean temporary test directory");
}

#[test]
fn cli_flow_list_filtered_by_tag() {
    let dir = unique_temp_dir();
    let first_task = "Learn Cobol #learn #core";
    let second_task = "Practice Assembler #code";
    let third_task = "Learn Rust #learn #important";

    let add_stdout = run_cli_stdout(&["add", first_task], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", second_task], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", third_task], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let mark_done_stdout = run_cli_stdout(&["mark-done", "Practice Assembler"], &dir, "mark-done");
    assert!(mark_done_stdout.contains("Successfully updated!"));

    let list_filtered_by_tag =
        run_cli_stdout(&["list", "--tag", "learn"], &dir, "list filtered by tags");

    assert_contains_task_in_section(&list_filtered_by_tag, "# To Do", "Learn Cobol");
    assert!(!list_filtered_by_tag.contains(second_task));

    fs::remove_dir_all(&dir).expect("Failed to clean temporary test directory");
}

#[test]
fn cli_flow_export_to_csv() {
    let dir = unique_temp_dir();
    let first_task = "Learn Cobol #learn #core";
    let second_task = "Practice \"try\" Assembler #code";
    let third_task = "Learn Rust #learn #important";

    let add_stdout = run_cli_stdout(&["add", first_task], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", second_task], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let add_stdout = run_cli_stdout(&["add", third_task], &dir, "add");
    assert!(add_stdout.contains("Successfully created!"));

    let list_exported = run_cli_stdout(&["export", "tasks.csv"], &dir, "export list to csv");

    let csv_path = dir.join("tasks.csv");

    let csv_content = fs::read_to_string(&csv_path).expect("Failed to read exported CSV file");

    assert!(list_exported.contains("exported"));
    assert!(csv_content.contains("name,status,priority,due_date,tags"));
    assert!(csv_content.contains("Learn Cobol"));
    assert!(csv_content.contains("Practice \"\"try\"\" Assembler"));
    assert!(csv_content.contains("core;learn"));

    fs::remove_dir_all(&dir).expect("Failed to clean temporary test directory");
}

#[test]
fn cli_flow_undo_add() {
    let dir = unique_temp_dir();

    run_cli_stdout(&["add", "Task A"], &dir, "add");

    let undo_output = run_cli_stdout(&["undo"], &dir, "undo");
    assert!(undo_output.contains("Successfully undone!"));

    let list_output = run_cli_stdout(&["list"], &dir, "list");
    assert!(list_output.contains("No Tasks found."));
}
