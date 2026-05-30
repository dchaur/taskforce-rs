use std::collections::{HashMap, HashSet};
use std::fs;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::domain::{ListInfo, Priority, Status, TodoItem, UndoAction};
use crate::utils::{get_task_tags, remove_tags_from_name};

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoList {
    items: HashMap<String, TodoItem>,
    undo_action: Option<UndoAction>,
}

impl TodoList {
    pub fn new() -> TodoList {
        let items: HashMap<String, TodoItem> = HashMap::new();

        TodoList {
            items,
            undo_action: None,
        }
    }

    pub fn add( 
        &mut self,
        key: String,
        priority: Option<Priority>,
        due_date_str: Option<String>,
    ) -> Result<(), String> {
        let task_tags = get_task_tags(&key);
        let cleaned_name = remove_tags_from_name(&key);

        if cleaned_name.is_empty() {
            return Err("Error: Task name cannot be empty".to_string());
        }

        if self.items.contains_key(&cleaned_name) {
            return Err(format!(
                "The task '{}' already exist, you can edit it using the command: cargo run -- edit \"My new task\" \"My updated task\"",
                cleaned_name
            ));
        }

        let due_date = due_date_str
            .map(|date_string| {
                NaiveDate::parse_from_str(&date_string, "%Y-%m-%d").map_err(|_| {
                    format!(
                        "Error: '{}' is not a valid date. Use YYYY-MM-DD",
                        date_string
                    )
                })
            })
            .transpose()?;

        if let Some(date) = due_date {
            let today = Local::now().naive_local().date();

            if date < today {
                return Err("Error: Due date must be in the future.".to_string());
            }
        }

        let default_priority = priority.unwrap_or(Priority::Medium);

        self.undo_action = Some(UndoAction::Add {
            key: cleaned_name.clone(),
        });

        self.items.insert(
            cleaned_name.clone(),
            TodoItem {
                name: cleaned_name,
                status: Status::Todo,
                priority: default_priority,
                due_date,
                tags: task_tags,
            },
        );

        Ok(())
    }

    pub fn mark(&mut self, key: String, value: Status) -> Result<String, String> {
        let todo_item = self.items.get_mut(&key).ok_or(&key)?;

        match value {
            Status::Todo => {
                self.undo_action = Some(UndoAction::MarkTodo {
                    key: key.clone(),
                    previous_item: todo_item.clone(),
                });
            }
            Status::Done => {
                self.undo_action = Some(UndoAction::MarkDone {
                    key: key.clone(),
                    previous_item: todo_item.clone(),
                });
            }
        }

        todo_item.status = value;

        Ok(key)
    }

    pub fn list(
        &self,
    ) -> (
        impl Iterator<Item = ListInfo> + '_,
        impl Iterator<Item = ListInfo> + '_,
    ) {
        (
            self.items
                .iter()
                .filter(|(_, todo_item)| todo_item.status == Status::Todo)
                .map(|(_, todo_item)| ListInfo {
                    name: todo_item.name.clone(),
                    priority: todo_item.priority,
                    due_date: todo_item.due_date,
                    tags: todo_item.tags.clone(),
                }),
            self.items
                .iter()
                .filter(|(_, todo_item)| todo_item.status == Status::Done)
                .map(|(_, todo_item)| ListInfo {
                    name: todo_item.name.clone(),
                    priority: todo_item.priority,
                    due_date: todo_item.due_date,
                    tags: todo_item.tags.clone(),
                }),
        )
    }

    pub fn load(path: &str) -> TodoList {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_else(TodoList::new)
    }

    pub fn save(&self, path: &str) {
        let contents = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, contents).unwrap();
    }

    pub fn remove(&mut self, key: String) -> Result<String, String> {
        let todo_item = self.items.get_mut(&key).ok_or(&key)?;
        self.undo_action = Some(UndoAction::Delete {
            key: key.clone(),
            previous_item: todo_item.clone(),
        });

        self.items.remove(&key).ok_or_else(|| key.clone())?;
        Ok(key)
    }

    pub fn status(&self, key: &str) -> Option<Status> {
        self.items.get(key).map(|item| item.status)
    }

    pub fn priority(&self, key: &str) -> Option<Priority> {
        self.items.get(key).map(|item| item.priority)
    }

    pub fn due_date(&self, key: &str) -> Option<NaiveDate> {
        self.items.get(key).and_then(|item| item.due_date)
    }

    pub fn tags(&self, key: &str) -> Option<HashSet<String>> {
        self.items.get(key).map(|item| item.tags.clone())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear_all(&mut self) -> usize {
        let original_len = self.items.len();
        self.items.clear();
        original_len - self.items.len()
    }

    pub fn clear_done(&mut self) -> usize {
        let original_len = self.items.len();
        self.items
            .retain(|_, todo_item| todo_item.status == Status::Todo);
        original_len - self.items.len()
    }

    pub fn clear_todo(&mut self) -> usize {
        let original_len = self.items.len();
        self.items
            .retain(|_, todo_item| todo_item.status == Status::Done);
        original_len - self.items.len()
    }

    pub fn edit(&mut self, old_key: String, new_key: String) -> Result<String, String> {
        if old_key != new_key && self.items.contains_key(&new_key) {
            return Err(format!("Task '{}' already exists", new_key));
        }

        let mut todo_item = self
            .items
            .remove(&old_key)
            .ok_or_else(|| format!("Task '{}' not found", old_key))?;

        let previous_item = todo_item.clone();

        todo_item.name = new_key.clone();

        self.undo_action = Some(UndoAction::Edit {
            key: old_key,
            new_key: new_key.clone(),
            previous_item,
        });

        self.items.insert(new_key.clone(), todo_item);

        Ok(new_key)
    }

    pub fn count(&self) -> (usize, usize, usize) {
        let total = self.items.len();
        let total_todo = self
            .items
            .values()
            .filter(|todo_item| todo_item.status == Status::Todo)
            .count();
        let total_done = self
            .items
            .values()
            .filter(|todo_item| todo_item.status == Status::Done)
            .count();

        (total, total_todo, total_done)
    }

    pub fn find(&self, keyword: &str) -> (Vec<ListInfo>, Vec<ListInfo>) {
        let keyword_lower = keyword.to_lowercase();
        let mut todos = Vec::new();
        let mut dones = Vec::new();

        for (_, item) in &self.items {
            if item.name.to_lowercase().contains(&keyword_lower) {
                let info = ListInfo {
                    name: item.name.clone(),
                    priority: item.priority,
                    due_date: item.due_date,
                    tags: item.tags.clone(),
                };

                match item.status {
                    Status::Todo => todos.push(info),
                    Status::Done => dones.push(info),
                }
            }
        }

        (todos, dones)
    }

    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        csv.push_str("name,status,priority,due_date,tags\n");

        for (_, item) in &self.items {
            let mut tags_str = String::new();

            if !item.tags.is_empty() {
                let mut tags: Vec<&str> = item.tags.iter().map(|s| s.as_str()).collect();
                tags.sort();

                tags_str = tags.join(";");
            }

            let due_date_str = item.due_date.map(|d| d.to_string()).unwrap_or_default();
            let escaped_name = item.name.replace('"', "\"\"");
            let status_lower_case = format!("{:?}", item.status).to_lowercase();
            let priority_lower_case = format!("{:?}", item.priority).to_lowercase();

            csv.push_str(&format!(
                "\"{}\",{},{},{},{}\n",
                escaped_name, status_lower_case, priority_lower_case, due_date_str, tags_str
            ));
        }

        csv
    }

    pub fn export(&self, path: Option<&str>) -> Result<(), String> {
        let csv_content = self.to_csv();

        match path {
            Some(filename) => {
                if std::path::Path::new(filename).exists() {
                    return Err(format!("Error: File '{}' already exists", filename));
                }

                std::fs::write(filename, csv_content)
                    .map_err(|e| format!("Error writing file: {}", e))?;
                Ok(())
            }
            None => {
                print!("{}", csv_content);
                Ok(())
            }
        }
    }

    pub fn undo(&mut self) -> Result<(), String> {
        let action = match self.undo_action.take() {
            Some(a) => a,
            None => return Err("Nothing to undo!".to_string()),
        };

        match action {
            UndoAction::Add { key } => {
                self.items.remove(&key);
            }
            UndoAction::Edit {
                key,
                new_key,
                previous_item,
            } => {
                self.items.remove(&new_key);
                self.items.insert(key, previous_item);
            }
            UndoAction::MarkTodo { key, previous_item }
            | UndoAction::MarkDone { key, previous_item } => {
                self.items.insert(key, previous_item);
            }
            UndoAction::Delete { key, previous_item } => {
                self.items.insert(key, previous_item);
            }
        }

        self.undo_action = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    struct TempFile<'a> {
        path: &'a str,
    }

    impl<'a> Drop for TempFile<'a> {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(self.path);
        }
    }

    #[test]
    fn init_todo() {
        let todo = TodoList::new();

        assert!(todo.is_empty());
    }

    #[test]
    fn add_item() {
        let fake_todo = "Something to do";
        let mut todo = TodoList::new();

        todo.add(fake_todo.into(), None, None)
            .expect("Should add item");

        assert_eq!(todo.status(fake_todo), Some(Status::Todo));
        assert_eq!(todo.due_date(fake_todo), None);
    }

    #[test]
    fn add_item_with_due_date() {
        let fake_todo = "Something to break";
        let mut todo = TodoList::new();

        todo.add(
            fake_todo.into(),
            Some(Priority::High),
            Some("2026-12-24".into()),
        )
        .expect("Should add item");

        assert_eq!(todo.status(fake_todo), Some(Status::Todo));
        assert_eq!(todo.priority(fake_todo), Some(Priority::High));
        assert_eq!(
            todo.due_date(fake_todo),
            Some(NaiveDate::from_ymd_opt(2026, 12, 24).unwrap())
        )
    }

    #[test]
    fn add_item_with_due_date_not_in_future() {
        let fake_todo = "Something to break";
        let mut todo = TodoList::new();
        let today = Local::now().naive_local().date();
        let yesterday = today - Duration::days(1);

        let result = todo.add(
            fake_todo.into(),
            Some(Priority::High),
            Some(yesterday.to_string()),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("future"));
        assert_eq!(todo.len(), 0);
    }

    #[test]
    fn add_item_custom_priority() {
        let fake_todo = "Something else to do";
        let mut todo = TodoList::new();

        todo.add(fake_todo.into(), Some(Priority::High), None)
            .expect("Should add item");

        assert_eq!(todo.status(fake_todo), Some(Status::Todo));
        assert_eq!(todo.priority(fake_todo), Some(Priority::High));
    }

    #[test]
    fn add_items_with_tags() {
        let task_name = "Walk my turtle #challenge #important";
        let cleaned_name = "Walk my turtle";
        let mut todo = TodoList::new();

        todo.add(task_name.into(), None, None)
            .expect("Should add item");

        // Name should be cleaned (tags removed)
        assert_eq!(todo.status(cleaned_name), Some(Status::Todo));
        assert_eq!(todo.status(task_name), None);

        // Tags should be extracted and stored
        let tags = todo.tags(cleaned_name).unwrap();
        assert!(tags.contains("important"));
        assert!(tags.contains("challenge"));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn add_task_with_only_tags_fails() {
        let mut todo = TodoList::new();
        let result = todo.add("#work #urgent".into(), None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
        assert_eq!(todo.len(), 0);
    }

    #[test]
    fn add_duplicate_task_with_different_tags_fails() {
        let mut todo = TodoList::new();

        todo.add("Task #bdd".into(), None, None).unwrap();
        let result = todo.add("Task #tdd".into(), None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exist"));
    }

    #[test]
    fn add_item_already_exist() {
        let fake_todo = "Something to do";
        let mut todo = TodoList::new();

        todo.add(fake_todo.into(), None, None).unwrap();
        let result = todo.add(fake_todo.into(), None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exist"));
        assert_eq!(todo.len(), 1);
    }

    #[test]
    fn add_item_does_not_change_value() {
        let fake_todo = "Something to do";
        let mut todo: TodoList = TodoList::new();
        todo.add(fake_todo.into(), None, None)
            .expect("Should add item");

        if let Some(x) = todo.items.get_mut(fake_todo) {
            x.status = Status::Done;
        }

        todo.add(fake_todo.into(), None, None)
            .expect_err("Should fail adding an item");

        assert_eq!(todo.status(fake_todo), Some(Status::Done));
        assert_eq!(todo.len(), 1);
    }

    #[test]
    fn mark_item() {
        let fake_todo = "Something to do";
        let mut todo = TodoList::new();

        todo.add(fake_todo.into(), None, None)
            .expect("Should add item");

        todo.mark(fake_todo.into(), Status::Done).unwrap();
        assert_eq!(todo.status(fake_todo), Some(Status::Done));

        todo.mark(fake_todo.into(), Status::Todo).unwrap();
        assert_eq!(todo.status(fake_todo), Some(Status::Todo));
    }

    #[test]
    fn mark_item_does_not_exist() {
        let fake_todo = "Something to do";
        let mut todo = TodoList::new();

        assert_eq!(
            todo.mark(fake_todo.into(), Status::Todo),
            Err(fake_todo.into())
        );
    }

    #[test]
    fn list_items() {
        let mut todo = TodoList::new();
        todo.add("Something to do".into(), None, None)
            .expect("Should add item");
        todo.add("Something else to do".into(), None, None)
            .expect("Should add item");
        todo.add("Something done".into(), None, None)
            .expect("Should add item");
        todo.mark("Something done".into(), Status::Done).unwrap();

        let (todo_iter, done_iter) = todo.list();

        let todo_items: Vec<ListInfo> = todo_iter.collect();
        let done_items: Vec<ListInfo> = done_iter.collect();

        assert!(todo_items.iter().any(|e| e.name == "Something to do"));
        assert!(todo_items.iter().any(|e| e.name == "Something else to do"));
        assert_eq!(todo_items.len(), 2);
        assert!(done_items.iter().any(|e| e.name == "Something done"));
        assert_eq!(done_items.len(), 1);
    }

    #[test]
    fn load_a_saved_todo_list() {
        let path = "test_load_todos.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();
        todo.add("Test item".to_string(), None, None)
            .expect("Should add item");
        todo.save(path);

        let loaded = TodoList::load(path);
        assert_eq!(loaded.status("Test item"), Some(Status::Todo));
    }

    #[test]
    fn remove_item() {
        let path = "test_remove_todos.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();
        todo.add("Test item".to_string(), None, None)
            .expect("Should add item");
        todo.save(path);

        let mut loaded = TodoList::load(path);

        assert_eq!(loaded.remove("Test item".into()), Ok("Test item".into()));
    }

    #[test]
    fn remove_item_does_not_exist() {
        let fake_todo = "I'm not in the list";
        let mut todo = TodoList::new();

        assert_eq!(todo.remove(fake_todo.into()), Err(fake_todo.into()));
    }

    #[test]
    fn clear_all_removes_all_items() {
        let mut todo = TodoList::new();
        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.mark("Beta".into(), Status::Done).unwrap();

        let removed_all = todo.clear_all();
        assert_eq!(removed_all, 2);
    }

    #[test]
    fn clear_all_no_items() {
        let mut todo = TodoList::new();

        let removed_all = todo.clear_all();
        assert_eq!(removed_all, 0);
    }

    #[test]
    fn clear_all_removes_all_items_from_storage() {
        let path = "test_clear_all.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.mark("Beta".into(), Status::Done).unwrap();
        todo.save(path);

        let mut loaded = TodoList::load(path);
        assert_eq!(loaded.status("Alpha"), Some(Status::Todo));

        let removed_stored_list = loaded.clear_all();
        assert_eq!(removed_stored_list, 2);
    }

    #[test]
    fn clear_done_removes_only_done_items() {
        let mut todo = TodoList::new();
        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.add("Delta".into(), None, None)
            .expect("Should add item");
        todo.mark("Delta".into(), Status::Done).unwrap();

        let removed = todo.clear_done();

        assert_eq!(removed, 1);
        assert_eq!(todo.len(), 2);
        assert_eq!(todo.status("Alpha"), Some(Status::Todo));
        assert_eq!(todo.status("Beta"), Some(Status::Todo));
        assert_eq!(todo.status("Delta"), None);
    }

    #[test]
    fn clear_done_no_items() {
        let mut todo = TodoList::new();
        let removed = todo.clear_done();

        assert_eq!(removed, 0);
        assert_eq!(todo.len(), 0);
    }

    #[test]
    fn clear_todo_removes_only_todo_items() {
        let mut todo = TodoList::new();
        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.add("Delta".into(), None, None)
            .expect("Should add item");
        todo.mark("Delta".into(), Status::Done).unwrap();

        let removed = todo.clear_todo();

        assert_eq!(removed, 2);
        assert_eq!(todo.len(), 1);
        assert_eq!(todo.status("Alpha"), None);
        assert_eq!(todo.status("Beta"), None);
        assert_eq!(todo.status("Delta"), Some(Status::Done));
    }

    #[test]
    fn clear_todo_no_items() {
        let mut todo = TodoList::new();
        let removed = todo.clear_done();

        assert_eq!(removed, 0);
        assert_eq!(todo.len(), 0);
    }

    #[test]
    fn edit_item_renames_key_preserves_status() {
        let mut todo = TodoList::new();
        todo.add("Old task".into(), None, None)
            .expect("Should add item");

        let result = todo.edit("Old task".into(), "New task".into());

        assert_eq!(result, Ok("New task".into()));
        assert_eq!(todo.status("Old task"), None);
        assert_eq!(todo.status("New task"), Some(Status::Todo));
        assert_eq!(todo.len(), 1);
    }

    #[test]
    fn edit_item_old_key_does_not_exist() {
        let mut todo = TodoList::new();
        todo.add("Existing task".into(), None, None)
            .expect("Should add item");

        let result = todo.edit("Non-existent".into(), "New name".into());

        assert_eq!(result, Err("Task 'Non-existent' not found".into()));
        assert_eq!(todo.len(), 1);
    }

    #[test]
    fn edit_item_preserves_done_status() {
        let mut todo = TodoList::new();
        let item = "Task";

        todo.add(item.into(), None, None).expect("Should add item");
        todo.mark(item.into(), Status::Done).unwrap();

        let result = todo.edit(item.into(), "Renamed task".into());

        assert_eq!(result, Ok("Renamed task".into()));
        assert_eq!(todo.status("Renamed task"), Some(Status::Done));
    }

    #[test]
    fn edit_item_new_key_already_exists() {
        let mut todo = TodoList::new();
        todo.add("Task A".into(), None, None)
            .expect("Should add item");
        todo.add("Task B".into(), None, None)
            .expect("Should add item");

        let result = todo.edit("Task A".into(), "Task B".into());

        assert_eq!(result, Err("Task 'Task B' already exists".into()));
        assert_eq!(todo.len(), 2);
        assert_eq!(todo.status("Task A"), Some(Status::Todo));
        assert_eq!(todo.status("Task B"), Some(Status::Todo));
    }

    #[test]
    fn count_returns_total_items_grouped() {
        let mut todo = TodoList::new();
        todo.add("Task A".into(), None, None)
            .expect("Should add item");
        todo.add("Task B".into(), None, None)
            .expect("Should add item");
        todo.add("Task C".into(), None, None)
            .expect("Should add item");

        todo.add("Task D".into(), None, None)
            .expect("Should add item");
        todo.mark("Task D".into(), Status::Done).unwrap();
        todo.add("Task E".into(), None, None)
            .expect("Should add item");
        todo.mark("Task E".into(), Status::Done).unwrap();

        let result = todo.count();
        assert_eq!(result, (5, 3, 2));
    }

    #[test]
    fn search_filters_items() {
        let mut todo = TodoList::new();
        todo.add("Learn Rust".into(), None, None)
            .expect("Should add item");

        todo.add("Walk my fish".into(), None, None)
            .expect("Should add item");

        todo.add("Practice Rust".into(), None, None)
            .expect("Should add item");

        todo.add("Collect gold".into(), None, None)
            .expect("Should add item");
        todo.mark("Collect gold".into(), Status::Done).unwrap();

        todo.add("Tattooing Rust in my arm".into(), None, None)
            .expect("Should add item");
        todo.mark("Tattooing Rust in my arm".into(), Status::Done)
            .unwrap();

        let (todos, dones) = todo.find("Rust");
        assert_eq!(todos.len(), 2);
        assert_eq!(dones.len(), 1);
    }

    #[test]
    fn export_to_file_creates_csv() {
        let path = "test_export_basic.csv";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        let today = Local::now().naive_local().date();
        let tomorrow = today + Duration::days(1);
        todo.add(
            "Task A".into(),
            Some(Priority::High),
            Some(tomorrow.to_string()),
        )
        .unwrap();

        todo.add("Task B #work".into(), None, None).unwrap();

        todo.export(Some(path)).unwrap();

        let content = std::fs::read_to_string(path).unwrap();

        assert!(content.contains("name,status,priority,due_date,tags"));

        assert!(content.contains("Task A"));
        assert!(content.contains("todo"));
        assert!(content.contains("high"));
    }

    #[test]
    fn export_file_already_exists_returns_error() {
        let path = "test_export_exist.csv";
        let _cleanup = TempFile { path };

        std::fs::write(path, "existing_data").unwrap();

        let todo = TodoList::new();
        let result = todo.export(Some(path));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn export_tags_semicolon_separated() {
        let path = "text_export_tags.csv";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        todo.add("Task #work #urgent #important".into(), None, None)
            .unwrap();

        todo.export(Some(path)).unwrap();

        let content = std::fs::read_to_string(path).unwrap();

        assert!(
            content.contains("important;urgent;work") || content.contains("work;urgent;important")
        );
    }

    #[test]
    fn export_empty_fields() {
        let path = "text_export_empty.csv";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        todo.add("Simple task".into(), None, None).unwrap();

        todo.export(Some(path)).unwrap();

        let content = std::fs::read_to_string(path).unwrap();

        assert!(content.contains("\"Simple task\",todo,medium,,"));
    }

    #[test]
    fn export_escapes_quotes_in_name() {
        let path = "test_export_quotes.csv";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        todo.add("Task with \"quotes\"".into(), None, None).unwrap();

        todo.export(Some(path)).unwrap();

        let content = std::fs::read_to_string(path).unwrap();

        assert!(content.contains("\"Task with \"\"quotes\"\"\""));
    }

    #[test]
    fn export_to_stdout_returns_csv_string() {
        let mut todo = TodoList::new();
        todo.add("Task A".into(), None, None).unwrap();

        let csv = todo.to_csv();

        assert!(csv.contains("name,status,priority,due_date,tags"));
        assert!(csv.contains("Task A"));
    }

    #[test]
    fn undo_add_removes_item() {
        let mut todo = TodoList::new();
        let fake_todo = "Task A";

        todo.add(fake_todo.into(), None, None).unwrap();

        let result = todo.undo();

        assert!(result.is_ok());
        assert_eq!(todo.len(), 0);
    }

    #[test]
    fn undo_edit_restores_original() {
        let mut todo = TodoList::new();
        let fake_todo = "Task A";

        todo.add(fake_todo.into(), None, None)
            .expect("Should add item");

        let result = todo.edit(fake_todo.into(), "Task B".into());

        assert_eq!(result, Ok("Task B".into()));
        assert_eq!(todo.status(fake_todo), None);
        assert_eq!(todo.status("Task B"), Some(Status::Todo));
        assert_eq!(todo.len(), 1);

        let undo_result = todo.undo();
        assert!(undo_result.is_ok());
        assert_eq!(todo.status(fake_todo), Some(Status::Todo));
        assert_eq!(todo.len(), 1);
    }

    #[test]
    fn undo_mark_done_restores_todo_status() {
        let fake_todo = "Something to do";
        let mut todo = TodoList::new();

        todo.add(fake_todo.into(), None, None)
            .expect("Should add item");

        todo.mark(fake_todo.into(), Status::Done).unwrap();

        assert_eq!(todo.status(fake_todo), Some(Status::Done));

        todo.undo().expect("Undo mark as done action");
        assert_eq!(todo.status(fake_todo), Some(Status::Todo));
    }

    #[test]
    fn undo_mark_todo_restores_done_status() {
        let fake_todo = "Something to do";
        let mut todo = TodoList::new();

        todo.add(fake_todo.into(), None, None)
            .expect("Should add item");

        todo.mark(fake_todo.into(), Status::Done).unwrap();
        assert_eq!(todo.status(fake_todo), Some(Status::Done));

        todo.mark(fake_todo.into(), Status::Todo).unwrap();
        assert_eq!(todo.status(fake_todo), Some(Status::Todo));

        todo.undo().expect("Undo mark as todo action");
        assert_eq!(todo.status(fake_todo), Some(Status::Done));
    }

    #[test]
    fn undo_delete_restores_item() {
        let path = "test_restore_removed_todo.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();
        todo.add("Test item".to_string(), None, None)
            .expect("Should add item");
        todo.save(path);

        let mut loaded_item = TodoList::load(path);

        assert_eq!(
            loaded_item.remove("Test item".into()),
            Ok("Test item".into())
        );

        let result = loaded_item.undo();

        assert!(result.is_ok());
        assert_eq!(loaded_item.len(), 1);
    }

    #[test]
    fn undo_without_previous_action_returns_error() {
        let mut todo = TodoList::new();
        let result = todo.undo();

        assert_eq!(result, Err("Nothing to undo!".to_string()));
    }

    #[test]
    fn undo_twice_returns_error() {
        let mut todo = TodoList::new();

        todo.add("Task".into(), None, None).unwrap();

        todo.undo().unwrap();

        let result = todo.undo();

        assert_eq!(result, Err("Nothing to undo!".to_string()));
    }

    #[test]
    fn new_operation_overwrites_previous_undo() {
        let mut todo = TodoList::new();

        todo.add("Task A".into(), None, None).unwrap();
        todo.add("Task B".into(), None, None).unwrap();

        todo.undo().unwrap();

        assert_eq!(todo.len(), 1); // Task A remains
        assert_eq!(todo.status("Task A"), Some(Status::Todo));
        assert_eq!(todo.status("Task B"), None);
    }
}
