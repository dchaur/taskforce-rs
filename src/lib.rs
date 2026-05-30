mod commands;
mod domain;
mod formatting;
mod interactive;
mod todo_list;
mod tui;
mod utils;

pub const TODO_FILE: &str = "todos.json";

pub use commands::{CommandOutcome, handle_command};
pub use domain::{ListInfo, Priority, Status};
pub use formatting::{
    format_outcome, format_section, get_priority_icon, get_task_prefix, pluralize,
};
pub use interactive::run_interactive_mode;
pub use todo_list::TodoList;
pub use tui::run_tui_mode;

#[cfg(test)]
#[ctor::ctor]
fn init_tests() {
    colored::control::set_override(false);
}
