use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

use crate::{
    ListInfo, Priority, Status, TODO_FILE, TodoList, format_outcome, handle_command,
    utils::prompt_select,
};

fn mark_task(
    theme: &ColorfulTheme,
    todo: &mut TodoList,
    command: &str,
    prompt: &str,
    items: Vec<ListInfo>,
) -> Result<(), String> {
    let task_names: Vec<String> = items.into_iter().map(|item| item.name).collect();

    if task_names.is_empty() {
        println!("\nNo todo tasks to mark as done.\n");
        return Ok(());
    }

    let task_selection = prompt_select(&theme, prompt, &task_names, 0)?;

    match handle_command(
        todo,
        command,
        Some(vec![task_names[task_selection].clone()]),
        None,
        None,
        None,
        None,
        TODO_FILE,
    ) {
        Ok(outcome) => println!("{}", format_outcome(outcome)),
        Err(e) => println!("\n✗ Error: {}\n", e),
    }

    Ok(())
}

pub fn run_interactive_mode() -> Result<(), String> {
    let mut todo = TodoList::load(TODO_FILE);
    let theme = ColorfulTheme::default();

    loop {
        let options = vec![
            "Add a new task",
            "List tasks",
            "Mark task as done",
            "Mark task as todo",
            "Edit task",
            "Remove task",
            "Count tasks",
            "Search",
            "Export",
            "Undo",
            "Exit",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(0)
            .interact()
            .map_err(|e| format!("Selection failed: {}", e))?;

        match selection {
            0 => {
                let task_name: String = Input::with_theme(&theme)
                    .with_prompt("Task name")
                    .interact_text()
                    .map_err(|e| format!("Input failed: {}", e))?;

                let priority_options = vec![Priority::Low, Priority::Medium, Priority::High];
                let mut priority_labels: Vec<String> =
                    priority_options.iter().map(|p| p.to_string()).collect();

                priority_labels.push(format!("Default ({})", Priority::Medium));

                let priority_selection = prompt_select(&theme, "Priority", &priority_labels, 3)?;
                let priority = priority_options.get(priority_selection).copied();
                let due_date_input: String = Input::with_theme(&theme)
                    .with_prompt("Due date (YYYY-MM-DD, or press Enter to skip)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| format!("Input failed: {}", e))?;

                let due_date = (!due_date_input.is_empty()).then(|| due_date_input);

                match handle_command(
                    &mut todo,
                    "add",
                    Some(vec![task_name]),
                    None,
                    priority,
                    due_date,
                    None,
                    TODO_FILE,
                ) {
                    Ok(outcome) => println!("{}", format_outcome(outcome)),
                    Err(e) => println!("\n✗ Error: {}\n", e),
                }
            }
            1 => {
                let filter_options = vec!["All tasks", "Todo only", "Done only"];
                let filter_selection =
                    prompt_select(&theme, "Filter by status", &filter_options, 0)?;
                let status_filter = match filter_selection {
                    1 => Some(Status::Todo),
                    2 => Some(Status::Done),
                    _ => None,
                };

                let tag_input: String = Input::with_theme(&theme)
                    .with_prompt("Filter by tags (comma-separated, or Enter to Skip)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| format!("Input failed: {}", e))?;

                let tag_filter = (!tag_input.is_empty()).then(|| vec![tag_input]);

                match handle_command(
                    &mut todo,
                    "list",
                    None,
                    status_filter,
                    None,
                    None,
                    tag_filter,
                    TODO_FILE,
                ) {
                    Ok(outcome) => println!("{}", format_outcome(outcome)),
                    Err(e) => println!("\n✗ Error: {}\n", e),
                }
            }
            2 => {
                let (todo_items, _) = todo.list();
                let items: Vec<ListInfo> = todo_items.collect();
                mark_task(
                    &theme,
                    &mut todo,
                    "mark-done",
                    "Select task to mark as done",
                    items,
                )?;
            }
            3 => {
                let (_, done_items) = todo.list();
                let items: Vec<ListInfo> = done_items.collect();
                mark_task(
                    &theme,
                    &mut todo,
                    "mark-todo",
                    "Select task to mark as todo",
                    items,
                )?;
            }
            4 => {
                let (todo_items, done_items) = todo.list();
                let task_names: Vec<String> =
                    todo_items.chain(done_items).map(|item| item.name).collect();

                let task_selection =
                    prompt_select(&theme, "Select task you want to edit", &task_names, 0)?;

                let new_task_name: String = Input::with_theme(&theme)
                    .with_prompt("New task name")
                    .interact_text()
                    .map_err(|e| format!("Input failed: {}", e))?;

                match handle_command(
                    &mut todo,
                    "edit",
                    Some(vec![task_names[task_selection].clone(), new_task_name]),
                    None,
                    None,
                    None,
                    None,
                    TODO_FILE,
                ) {
                    Ok(outcome) => println!("{}", format_outcome(outcome)),
                    Err(e) => println!("\n✗ Error: {}\n", e),
                }
            }
            5 => {
                let (todo_items, done_items) = todo.list();
                let task_names: Vec<String> =
                    todo_items.chain(done_items).map(|item| item.name).collect();

                let task_selection =
                    prompt_select(&theme, "Select the task you want to remove", &task_names, 0)?;

                let confirmation = Confirm::with_theme(&theme)
                    .with_prompt(format!(
                        "Are you sure you want to delete '{}'?",
                        task_names[task_selection]
                    ))
                    .interact()
                    .map_err(|e| format!("Confirmation failed: {}", e))?;

                if !confirmation {
                    println!("\n✓ Delete cancelled\n");
                    continue;
                }

                match handle_command(
                    &mut todo,
                    "remove",
                    Some(vec![task_names[task_selection].clone()]),
                    None,
                    None,
                    None,
                    None,
                    TODO_FILE,
                ) {
                    Ok(outcome) => println!("{}", format_outcome(outcome)),
                    Err(e) => println!("\n✗ Error: {}\n", e),
                }
            }
            6 => {
                match handle_command(&mut todo, "count", None, None, None, None, None, TODO_FILE) {
                    Ok(outcome) => println!("{}", format_outcome(outcome)),
                    Err(e) => println!("\n✗ Error: {}\n", e),
                }
            }
            7 => {
                let search: String = Input::with_theme(&theme)
                    .with_prompt("What are you looking for?")
                    .interact_text()
                    .map_err(|e| format!("Input failed: {}", e))?;

                match handle_command(
                    &mut todo,
                    "find",
                    Some(vec![search]),
                    None,
                    None,
                    None,
                    None,
                    TODO_FILE,
                ) {
                    Ok(outcome) => println!("{}", format_outcome(outcome)),
                    Err(e) => println!("\n✗ Error: {}\n", e),
                }
            }
            8 => {
                let csv_name: String = Input::with_theme(&theme)
                    .with_prompt("Filename (or Enter to display CSV)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| format!("Input failed: {}", e))?;

                let csv_selection = (!csv_name.is_empty()).then(|| vec![csv_name]);

                match handle_command(
                    &mut todo,
                    "export",
                    csv_selection,
                    None,
                    None,
                    None,
                    None,
                    TODO_FILE,
                ) {
                    Ok(outcome) => println!("{}", format_outcome(outcome)),
                    Err(e) => println!("\n✗ Error: {}\n", e),
                }
            }
            9 => {
                let confirmation = Confirm::with_theme(&theme)
                    .with_prompt("Are you sure you want to undo the last action?")
                    .interact()
                    .map_err(|e| format!("Confirmation failed: {}", e))?;

                if !confirmation {
                    println!("\n✓ Undo cancelled\n");
                    continue;
                }

                match handle_command(&mut todo, "undo", None, None, None, None, None, TODO_FILE) {
                    Ok(outcome) => println!("{}", format_outcome(outcome)),
                    Err(e) => println!("\n✗ Error: {}\n", e),
                }
            }
            10 => {
                println!("\nGoodbye! 👋\n");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
