use crate::domain::{ListInfo, Priority, Status};
use crate::todo_list::TodoList;

#[derive(Debug, PartialEq)]
pub enum CommandOutcome {
    Created,
    Updated,
    Listed {
        todo_items: Vec<ListInfo>,
        done_items: Vec<ListInfo>,
    },
    Counted {
        total: usize,
        total_todo: usize,
        total_done: usize,
    },
    Exported,
    Undone,
}

pub fn handle_command(
    todo: &mut TodoList,
    command: &str,
    key: Option<Vec<String>>,
    status: Option<Status>,
    priority: Option<Priority>,
    due: Option<String>,
    tag: Option<Vec<String>>,
    path: &str,
) -> Result<CommandOutcome, String> {
    if tag.is_some() && command != "list" {
        return Err(format!(
            "Error: --tag can only be used with the 'list' command, not '{}'",
            command
        ));
    }

    match command {
        "add" => {
            let keys = key.ok_or_else(|| "Key cannot be empty".to_string())?;
            let task = keys
                .get(0)
                .ok_or_else(|| "Key cannot be empty".to_string())?;

            todo.add(task.to_string(), priority, due)?;
            todo.save(path);
            Ok(CommandOutcome::Created)
        }
        "remove" => {
            let keys = key.ok_or_else(|| "Key cannot be empty".to_string())?;
            let task = keys
                .get(0)
                .ok_or_else(|| "Key cannot be empty".to_string())?;

            todo.remove(task.clone())
                .map_err(|e| format!("Invalid key {}", e))?;
            todo.save(path);
            Ok(CommandOutcome::Updated)
        }
        "mark-done" => {
            let keys = key.ok_or_else(|| "Key cannot be empty".to_string())?;
            let task = keys
                .get(0)
                .ok_or_else(|| "Key cannot be empty".to_string())?;

            todo.mark(task.to_string(), Status::Done)
                .map_err(|e| format!("Invalid key {}", e))?;
            todo.save(path);
            Ok(CommandOutcome::Updated)
        }
        "mark-todo" => {
            let keys = key.ok_or_else(|| "Key cannot be empty".to_string())?;
            let task = keys
                .get(0)
                .ok_or_else(|| "Key cannot be empty".to_string())?;

            todo.mark(task.to_string(), Status::Todo)
                .map_err(|e| format!("Invalid key {}", e))?;
            todo.save(path);
            Ok(CommandOutcome::Updated)
        }
        "list" => {
            let (todo_iter, done_iter) = todo.list();

            let (mut todo_items, mut done_items) = match status {
                Some(Status::Todo) => (todo_iter.collect(), vec![]),
                Some(Status::Done) => (vec![], done_iter.collect()),
                None => (todo_iter.collect(), done_iter.collect()),
            };

            let raw_tags = tag.iter().flatten();
            let filter_tags: Vec<String> = raw_tags
                .flat_map(|s| s.split(','))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            let is_filtering_untagged = filter_tags.iter().any(|t| t == "none");
            let actual_tags: Vec<String> =
                filter_tags.into_iter().filter(|t| t != "none").collect();

            if is_filtering_untagged && actual_tags.is_empty() {
                todo_items.retain(|item| item.tags.is_empty());
                done_items.retain(|item| item.tags.is_empty());
            } else if is_filtering_untagged && !actual_tags.is_empty() {
                todo_items.retain(|item| {
                    item.tags.is_empty() || actual_tags.iter().any(|ft| item.tags.contains(ft))
                });
                done_items.retain(|item| {
                    item.tags.is_empty() || actual_tags.iter().any(|ft| item.tags.contains(ft))
                });
            } else if !actual_tags.is_empty() {
                todo_items.retain(|item| actual_tags.iter().any(|ft| item.tags.contains(ft)));
                done_items.retain(|item| actual_tags.iter().any(|ft| item.tags.contains(ft)));
            }

            Ok(CommandOutcome::Listed {
                todo_items,
                done_items,
            })
        }
        "clear-all" => {
            todo.clear_all();
            todo.save(path);
            Ok(CommandOutcome::Updated)
        }
        "clear-done" => {
            todo.clear_done();
            todo.save(path);
            Ok(CommandOutcome::Updated)
        }
        "clear-todo" => {
            todo.clear_todo();
            todo.save(path);
            Ok(CommandOutcome::Updated)
        }
        "edit" => {
            let keys = key.ok_or_else(|| "Key cannot be empty".to_string())?;
            let old_key = keys.get(0).ok_or_else(|| "Old key required".to_string())?;
            let new_key = keys.get(1).ok_or_else(|| "New key required".to_string())?;

            todo.edit(old_key.clone(), new_key.clone())
                .map_err(|e| format!("Edit failed: {}", e))?;

            todo.save(path);

            Ok(CommandOutcome::Updated)
        }
        "count" => {
            let (total, total_todo, total_done) = todo.count();
            Ok(CommandOutcome::Counted {
                total,
                total_todo,
                total_done,
            })
        }
        "find" => {
            let keys = key.ok_or_else(|| "Search keyword cannot be empty".to_string())?;
            let keyword = keys
                .get(0)
                .ok_or_else(|| "Search keyword cannot be empty".to_string())?;
            let (todo_items, done_items) = todo.find(keyword);

            Ok(CommandOutcome::Listed {
                todo_items,
                done_items,
            })
        }
        "export" => {
            let filename = key
                .as_ref()
                .and_then(|keys| keys.get(0).map(|s| s.as_str()));

            todo.export(filename)?;

            Ok(CommandOutcome::Exported)
        }
        "undo" => {
            todo.undo()?;
            todo.save(path);

            Ok(CommandOutcome::Undone)
        }
        cmd => Err(format!("Command {} not recognized", cmd)),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;

    struct TempFile<'a> {
        path: &'a str,
    }

    impl<'a> Drop for TempFile<'a> {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(self.path);
        }
    }

    #[test]
    fn handle_command_tag_with_non_list_fails() {
        let mut todo = TodoList::new();
        let result = handle_command(
            &mut todo,
            "add",
            Some(vec!["work".to_string()]),
            None,
            None,
            None,
            Some(vec!["work".to_string()]),
            "unused.json",
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--tag can only be used with"));
    }

    #[test]
    fn handle_command_list_filtered_by_todo() {
        let mut todo = TodoList::new();
        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.add("Delta".into(), None, None)
            .expect("Should add item");
        todo.mark("Delta".into(), Status::Done).unwrap();

        let result = handle_command(
            &mut todo,
            "list",
            None,
            Some(Status::Todo),
            None,
            None,
            None,
            "unused.json",
        );

        match result {
            Ok(CommandOutcome::Listed {
                todo_items,
                done_items,
            }) => {
                assert_eq!(todo_items.len(), 2);
                assert!(todo_items.iter().any(|item| item.name == "Alpha"));
                assert_eq!(done_items.len(), 0);
            }
            _ => panic!("Expected Listed Outcome"),
        }
    }

    #[test]
    fn handle_command_list_filtered_by_done() {
        let mut todo = TodoList::new();
        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.add("Delta".into(), None, None)
            .expect("Should add item");
        todo.mark("Delta".into(), Status::Done).unwrap();

        let result = handle_command(
            &mut todo,
            "list",
            None,
            Some(Status::Done),
            None,
            None,
            None,
            "unused_2.json",
        );

        match result {
            Ok(CommandOutcome::Listed {
                todo_items,
                done_items,
            }) => {
                assert_eq!(todo_items.len(), 0);
                assert_eq!(done_items.len(), 1);
                assert!(done_items.iter().any(|item| item.name == "Delta"));
            }
            _ => panic!("Expected Listed Outcome"),
        }
    }

    #[test]
    fn handle_command_list_not_filter_shows_all() {
        let mut todo = TodoList::new();
        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.add("Delta".into(), None, None)
            .expect("Should add item");
        todo.mark("Delta".into(), Status::Done).unwrap();

        let result = handle_command(
            &mut todo,
            "list",
            None,
            None,
            None,
            None,
            None,
            "unused_2.json",
        );

        match result {
            Ok(CommandOutcome::Listed {
                todo_items,
                done_items,
            }) => {
                assert_eq!(todo_items.len(), 2);
                assert_eq!(done_items.len(), 1);
                assert!(done_items.iter().any(|item| item.name == "Delta"));
            }
            _ => panic!("Expected Listed Outcome"),
        }
    }

    #[test]
    fn handle_command_list_filtered_by_single_tag() {
        let mut todo = TodoList::new();
        todo.add("Task A #work".into(), None, None).unwrap();
        todo.add("Task B #personal".into(), None, None).unwrap();

        let result = handle_command(
            &mut todo,
            "list",
            None,
            None,
            None,
            None,
            Some(vec!["work".to_string()]),
            "unused_list_tag_filtered.json",
        );

        match result {
            Ok(CommandOutcome::Listed { todo_items, .. }) => {
                assert_eq!(todo_items.len(), 1);
                assert_eq!(todo_items[0].name, "Task A");
            }
            _ => panic!("Expected Listed outcome"),
        }
    }

    #[test]
    fn handle_command_list_filtered_by_tag_none_shows_untagged() {
        let mut todo = TodoList::new();
        todo.add("Task A #work".into(), None, None).unwrap();
        todo.add("Task B".into(), None, None).unwrap();

        let result = handle_command(
            &mut todo,
            "list",
            None,
            None,
            None,
            None,
            Some(vec!["none".to_string()]),
            "unused_list_tag_none.json",
        );

        match result {
            Ok(CommandOutcome::Listed { todo_items, .. }) => {
                assert_eq!(todo_items.len(), 1);
                assert_eq!(todo_items[0].name, "Task B");
            }
            _ => panic!("Expected Listed outcome"),
        }
    }

    #[test]
    fn handle_command_list_filtered_by_multiple_tags_or_logic() {
        let mut todo = TodoList::new();
        todo.add("Task A #work".into(), None, None).unwrap();
        todo.add("Task B #personal".into(), None, None).unwrap();
        todo.add("Task C #home".into(), None, None).unwrap();

        let result = handle_command(
            &mut todo,
            "list",
            None,
            None,
            None,
            None,
            Some(vec!["work,personal".to_string()]),
            "unused_list_tag_filtered.json",
        );

        match result {
            Ok(CommandOutcome::Listed { todo_items, .. }) => {
                assert_eq!(todo_items.len(), 2);
                assert!(todo_items.iter().any(|item| item.name.contains("Task A")));
            }
            _ => panic!("Expected Listed outcome"),
        }
    }

    #[test]
    fn handle_command_add_success() {
        let path = "test_handle_add_success.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        let result = handle_command(
            &mut todo,
            "add",
            Some(vec!["Test item".to_string()]),
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(result, Ok(CommandOutcome::Created));
        assert_eq!(todo.status("Test item"), Some(Status::Todo));
    }

    #[test]
    fn handle_command_add_with_due_date_success() {
        let path = "test_handle_add_success.json";
        let _cleanup = TempFile { path };
        let today = Local::now().naive_local().date();
        let mut todo = TodoList::new();

        let result = handle_command(
            &mut todo,
            "add",
            Some(vec!["Test item".to_string()]),
            None,
            None,
            Some(today.to_string()),
            None,
            path,
        );

        assert_eq!(result, Ok(CommandOutcome::Created));
        assert_eq!(todo.status("Test item"), Some(Status::Todo));
        assert_eq!(todo.due_date("Test item"), Some(today));
    }

    #[test]
    fn handle_command_add_with_priority_success() {
        let path = "test_handle_command_add_priority_success.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        let result = handle_command(
            &mut todo,
            "add",
            Some(vec!["Test item".to_string()]),
            None,
            Some(Priority::High),
            None,
            None,
            path,
        );

        assert_eq!(result, Ok(CommandOutcome::Created));
        assert_eq!(todo.status("Test item"), Some(Status::Todo));
        assert_eq!(todo.priority("Test item"), Some(Priority::High));
    }

    #[test]
    fn handle_command_mark_todo_success() {
        let path = "test_handle_mark_todo_success.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        let add_result = handle_command(
            &mut todo,
            "add",
            Some(vec!["Test item".to_string()]),
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(add_result, Ok(CommandOutcome::Created));
        assert_eq!(todo.status("Test item"), Some(Status::Todo));

        let mark_done_result = handle_command(
            &mut todo,
            "mark-done",
            Some(vec!["Test item".to_string()]),
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(mark_done_result, Ok(CommandOutcome::Updated));
        assert_eq!(todo.status("Test item"), Some(Status::Done));

        let mark_todo_result = handle_command(
            &mut todo,
            "mark-todo",
            Some(vec!["Test item".to_string()]),
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(mark_todo_result, Ok(CommandOutcome::Updated));
        assert_eq!(todo.status("Test item"), Some(Status::Todo));
    }

    #[test]
    fn handle_command_mark_todo_invalid_key() {
        let path = "test_handle_mark_todo_invalid_key.json";
        let mut todo = TodoList::new();
        let invalid_item = "Test wrong item";
        let expected_error = format!("Invalid key {}", invalid_item);

        todo.add("Test item".to_string(), None, None)
            .expect("Should add item");

        let result = handle_command(
            &mut todo,
            "mark-todo",
            Some(vec![invalid_item.to_string()]),
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(result, Err(expected_error));
    }

    #[test]
    fn handle_command_remove_success() {
        let path = "test_handle_remove_success.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();
        todo.add("Test item".to_string(), None, None)
            .expect("Should add item");

        let result = handle_command(
            &mut todo,
            "remove",
            Some(vec!["Test item".to_string()]),
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(result, Ok(CommandOutcome::Updated));
        assert_eq!(todo.status("Test item"), None);
    }

    #[test]
    fn handle_command_list_success() {
        let mut todo = TodoList::new();
        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.mark("Beta".into(), Status::Done).unwrap();

        let result = handle_command(
            &mut todo,
            "list",
            None,
            None,
            None,
            None,
            None,
            "unused.json",
        );

        match result {
            Ok(CommandOutcome::Listed {
                todo_items,
                done_items,
            }) => {
                assert!(todo_items.iter().any(|item| item.name == "Alpha"));
                assert!(done_items.iter().any(|item| item.name == "Beta"));
                assert_eq!(todo_items.len(), 1);
                assert_eq!(done_items.len(), 1);
            }
            _ => panic!("Expected list command to return Listed outcome"),
        }
    }

    #[test]
    fn handle_command_clear_all() {
        let path = "test_command_clear_all.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.mark("Beta".into(), Status::Done).unwrap();
        todo.add("Delta".into(), None, None)
            .expect("Should add item");
        todo.mark("Delta".into(), Status::Done).unwrap();

        let result = handle_command(&mut todo, "clear-all", None, None, None, None, None, path);

        assert_eq!(result, Ok(CommandOutcome::Updated));
        assert_eq!(todo.len(), 0);
    }

    #[test]
    fn handle_command_clear_done() {
        let path = "test_clear_done.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.mark("Beta".into(), Status::Done).unwrap();
        todo.add("Delta".into(), None, None)
            .expect("Should add item");
        todo.mark("Delta".into(), Status::Done).unwrap();

        let result = handle_command(&mut todo, "clear-done", None, None, None, None, None, path);

        assert_eq!(result, Ok(CommandOutcome::Updated));
        assert_eq!(todo.len(), 1);
    }

    #[test]
    fn handle_command_clear_todo() {
        let path = "test_clear_todo.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        todo.add("Alpha".into(), None, None)
            .expect("Should add item");
        todo.add("Beta".into(), None, None)
            .expect("Should add item");
        todo.mark("Beta".into(), Status::Done).unwrap();
        todo.add("Delta".into(), None, None)
            .expect("Should add item");
        todo.mark("Delta".into(), Status::Done).unwrap();

        let result = handle_command(&mut todo, "clear-todo", None, None, None, None, None, path);

        assert_eq!(result, Ok(CommandOutcome::Updated));
        assert_eq!(todo.len(), 2);
    }

    #[test]
    fn handle_command_edit_item() {
        let path = "test_edit_item.json";
        let _cleanup = TempFile { path };
        let mut todo = TodoList::new();

        todo.add("Alpha".into(), None, None)
            .expect("Should add item");

        let result = handle_command(
            &mut todo,
            "edit",
            Some(vec!["Alpha".to_string(), "Beta".to_string()]),
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(todo.status("Alpha"), None);
        assert_eq!(todo.status("Beta"), Some(Status::Todo));
        assert_eq!(result, Ok(CommandOutcome::Updated));
    }

    #[test]
    fn handle_command_count() {
        let path = "test_count_command.json";
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

        let result = handle_command(&mut todo, "count", None, None, None, None, None, path);

        match result {
            Ok(CommandOutcome::Counted {
                total,
                total_todo,
                total_done,
            }) => {
                assert_eq!(total, 5);
                assert_eq!(total_todo, 3);
                assert_eq!(total_done, 2);
            }
            _ => panic!("Expected list command to return Counted outcome"),
        }
    }

    #[test]
    fn handle_command_find() {
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

        let result = handle_command(
            &mut todo,
            "find",
            Some(vec!["Rust".to_string()]),
            None,
            None,
            None,
            None,
            "unused.json",
        );

        match result {
            Ok(CommandOutcome::Listed {
                todo_items,
                done_items,
            }) => {
                assert!(todo_items.iter().any(|item| item.name.contains("Rust")));
                assert!(done_items.iter().any(|item| item.name.contains("Rust")));
                assert_eq!(todo_items.len(), 2);
                assert_eq!(done_items.len(), 1);
            }
            _ => panic!("Expected list command to return Listed outcome"),
        }
    }

    #[test]
    fn handle_command_export_success() {
        let path = "text_handle_export.csv";
        let _cleanup = TempFile { path };

        let mut todo = TodoList::new();
        todo.add("Task A".into(), None, None).unwrap();

        let result = handle_command(
            &mut todo,
            "export",
            Some(vec![path.to_string()]),
            None,
            None,
            None,
            None,
            "unused.json",
        );

        assert!(result.is_ok());
        assert_eq!(result, Ok(CommandOutcome::Exported));
        assert!(std::path::Path::new(path).exists());
    }

    #[test]
    fn handle_command_undo() {
        let mut todo = TodoList::new();
        todo.add("Task A".into(), None, None).unwrap();

        let result = handle_command(
            &mut todo,
            "undo",
            None,
            None,
            None,
            None,
            None,
            "unused.json",
        );

        assert!(result.is_ok());
        assert_eq!(result, Ok(CommandOutcome::Undone));
    }

    #[test]
    fn handle_command_missing_key() {
        let path = "test_handle_remove_missing.json";
        let mut todo = TodoList::new();
        todo.add("Test item".to_string(), None, None)
            .expect("Should add item");

        let result = handle_command(&mut todo, "remove", None, None, None, None, None, path);

        assert_eq!(result, Err("Key cannot be empty".to_string()));
    }

    #[test]
    fn handle_command_invalid_key() {
        let path = "test_handle_remove_missing.json";
        let mut todo = TodoList::new();
        let invalid_item = "Test wrong item";
        let expected_error = format!("Invalid key {}", invalid_item);

        todo.add("Test item".to_string(), None, None)
            .expect("Should add item");

        let result = handle_command(
            &mut todo,
            "remove",
            Some(vec![invalid_item.to_string()]),
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(result, Err(expected_error));
    }

    #[test]
    fn handle_command_unknown_command() {
        let path = "test_handle_unknown_command.json";
        let unknown_command = "whatever";
        let expected_error = format!("Command {} not recognized", unknown_command);
        let mut todo = TodoList::new();

        let result = handle_command(
            &mut todo,
            unknown_command,
            None,
            None,
            None,
            None,
            None,
            path,
        );

        assert_eq!(result, Err(expected_error));
    }
}
