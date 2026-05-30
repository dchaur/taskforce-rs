use std::collections::HashSet;

use dialoguer::{Select, theme::ColorfulTheme};

pub fn get_task_tags(task: &str) -> HashSet<String> {
    task.split_whitespace()
        .filter(|word| word.starts_with('#'))
        .map(|tag| tag[1..].to_string())
        .collect()
}

pub fn remove_tags_from_name(task: &str) -> String {
    task.split_whitespace()
        .filter(|word| !word.starts_with('#'))
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn prompt_select<T: ToString>(
    theme: &ColorfulTheme,
    prompt: &str,
    items: &[T],
    default: usize,
) -> Result<usize, String> {
    Select::with_theme(theme)
        .with_prompt(prompt)
        .items(items)
        .default(default)
        .interact()
        .map_err(|e| format!("Selection failed: {}", e))
}
