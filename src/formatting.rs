use chrono::{Local, NaiveDate};
use colored::Colorize;

use crate::{CommandOutcome, ListInfo, Priority};

pub fn get_priority_icon(priority: Priority) -> String {
    match priority {
        Priority::High => "⬆️".to_string(),
        Priority::Medium => "➡️".to_string(),
        Priority::Low => "⬇️".to_string(),
    }
}

pub fn get_task_prefix(priority: Priority, due_date: Option<NaiveDate>) -> String {
    let today = Local::now().naive_local().date();

    match due_date {
        Some(date) if date < today => format!("{} {}", "🔴".red(), "OVERDUE:".red().bold()),
        _ => format!("{} ", get_priority_icon(priority)),
    }
}

pub fn format_section(title: &str, items: Vec<ListInfo>) -> String {
    let mut section = String::new();

    section.push('\n');
    section.push_str(&format!("# {}\n\n", title.blue().bold()));

    for item in items {
        let due_info = item
            .due_date
            .map(|date| format!("(due: {})", date))
            .unwrap_or_else(String::new);

        let tags_info = if !item.tags.is_empty() {
            let mut tags_vec: Vec<String> = item
                .tags
                .iter()
                .map(|t| format!("#{}", t.bright_yellow()))
                .collect();
            tags_vec.sort();
            format!(" [{}]", tags_vec.join(" "))
        } else {
            String::new()
        };

        section.push_str(&format!(
            " * {} {}{} {}\n",
            get_task_prefix(item.priority, item.due_date),
            item.name,
            tags_info,
            due_info,
        ));
    }

    section
}

pub fn pluralize(count: usize, singular: &str) -> String {
    if count != 1 {
        return format!("{}s", singular);
    }

    singular.to_string()
}

pub fn format_outcome(outcome: CommandOutcome) -> String {
    match outcome {
        CommandOutcome::Created => "✓ Successfully created!".green().to_string(),
        CommandOutcome::Updated => "✓ Successfully updated!".green().to_string(),
        CommandOutcome::Exported => "✓ Successfully exported!".green().to_string(),
        CommandOutcome::Undone => "✓ Successfully undone!".green().to_string(),
        CommandOutcome::Listed {
            todo_items,
            done_items,
        } => {
            if todo_items.is_empty() && done_items.is_empty() {
                return "\nNo Tasks found.\n".yellow().to_string();
            }

            let mut output = String::new();

            if !todo_items.is_empty() {
                output.push_str(&format_section("To Do", todo_items));
            }

            if !done_items.is_empty() {
                output.push_str(&format_section("Done", done_items));
            }
            output
        }
        CommandOutcome::Counted {
            total,
            total_todo,
            total_done,
        } => {
            let mut output = String::new();
            output.push('\n');
            output.push_str(&format!(
                "# Total: {} {}\n",
                total.to_string().cyan().bold(),
                pluralize(total, "task")
            ));
            output.push_str(&format!(
                "# To Do: {} {}\n",
                total_todo.to_string().cyan().bold(),
                pluralize(total_todo, "task")
            ));
            output.push_str(&format!(
                "# Done: {} {}\n",
                total_done.to_string().cyan().bold(),
                pluralize(total_done, "task")
            ));

            output
        }
    }
}

#[cfg(test)]

mod tests {
    use chrono::{Duration, Local};
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn format_outcome_created_message() {
        assert_eq!(
            format_outcome(CommandOutcome::Created),
            "✓ Successfully created!"
        );
    }

    #[test]
    fn format_outcome_updated_message() {
        assert_eq!(
            format_outcome(CommandOutcome::Updated),
            "✓ Successfully updated!"
        );
    }

    #[test]
    fn format_outcome_listed_message_contains_sections_and_items() {
        let formatted_outcome = format_outcome(CommandOutcome::Listed {
            todo_items: vec![
                ListInfo {
                    name: "A".into(),
                    priority: Priority::High,
                    due_date: None,
                    tags: HashSet::new(),
                },
                ListInfo {
                    name: "B".into(),
                    priority: Priority::Low,
                    due_date: None,
                    tags: HashSet::new(),
                },
            ],
            done_items: vec![
                ListInfo {
                    name: "C".into(),
                    priority: Priority::Medium,
                    due_date: None,
                    tags: HashSet::new(),
                },
                ListInfo {
                    name: "D".into(),
                    priority: Priority::High,
                    due_date: None,
                    tags: HashSet::new(),
                },
            ],
        });

        let expected = "\n# To Do\n\n * ⬆️  A \n * ⬇️  B \n\n# Done\n\n * ➡️  C \n * ⬆️  D \n";
        assert_eq!(formatted_outcome, expected);
    }

    #[test]
    fn format_outcome_listed_message_contains_due_date() {
        let today = Local::now().naive_local().date();
        let formatted_outcome = format_outcome(CommandOutcome::Listed {
            todo_items: vec![
                ListInfo {
                    name: "A".into(),
                    priority: Priority::High,
                    due_date: None,
                    tags: HashSet::new(),
                },
                ListInfo {
                    name: "B".into(),
                    priority: Priority::Low,
                    due_date: Some(today),
                    tags: HashSet::new(),
                },
            ],
            done_items: vec![],
        });

        let expected = format!("\n# To Do\n\n * ⬆️  A \n * ⬇️  B (due: {today})\n");
        assert_eq!(formatted_outcome, expected);
    }

    #[test]
    fn format_outcome_listed_message_contains_over_due_date() {
        let today = Local::now().naive_local().date();
        let yesterday = today - Duration::days(1);
        let formatted_outcome = format_outcome(CommandOutcome::Listed {
            todo_items: vec![
                ListInfo {
                    name: "A".into(),
                    priority: Priority::High,
                    due_date: None,
                    tags: HashSet::new(),
                },
                ListInfo {
                    name: "B".into(),
                    priority: Priority::Low,
                    due_date: Some(yesterday),
                    tags: HashSet::new(),
                },
            ],
            done_items: vec![],
        });

        let expected = format!("\n# To Do\n\n * ⬆️  A \n * 🔴 OVERDUE: B (due: {yesterday})\n");
        assert_eq!(formatted_outcome, expected);
    }

    #[test]
    fn empty_todo_items_outcome_should_not_listed() {
        let formatted_outcome = format_outcome(CommandOutcome::Listed {
            todo_items: vec![],
            done_items: vec![
                ListInfo {
                    name: "A".into(),
                    priority: Priority::High,
                    due_date: None,
                    tags: HashSet::new(),
                },
                ListInfo {
                    name: "B".into(),
                    priority: Priority::Medium,
                    due_date: None,
                    tags: HashSet::new(),
                },
            ],
        });

        let expected = "\n# Done\n\n * ⬆️  A \n * ➡️  B \n";
        assert_eq!(formatted_outcome, expected);
    }

    #[test]
    fn empty_done_items_outcome_should_not_listed() {
        let formatted_outcome = format_outcome(CommandOutcome::Listed {
            todo_items: vec![
                ListInfo {
                    name: "A".into(),
                    priority: Priority::High,
                    due_date: None,
                    tags: HashSet::new(),
                },
                ListInfo {
                    name: "B".into(),
                    priority: Priority::Low,
                    due_date: None,
                    tags: HashSet::new(),
                },
            ],
            done_items: vec![],
        });

        let expected = "\n# To Do\n\n * ⬆️  A \n * ⬇️  B \n";
        assert_eq!(formatted_outcome, expected);
    }
}
