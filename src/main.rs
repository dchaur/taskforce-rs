use clap::Parser;

use todo_cli::{
    Priority, Status, TODO_FILE, TodoList, format_outcome, handle_command, run_interactive_mode,
    run_tui_mode,
};

#[derive(Parser)]
#[command(version)]
struct Cli {
    command: String,
    #[arg(num_args = 1..)]
    key: Option<Vec<String>>,
    #[arg(long)]
    status: Option<String>,
    #[arg(long)]
    priority: Option<String>,
    #[arg(long)]
    due: Option<String>,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    tag: Option<Vec<String>>,
}

fn main() {
    let args = Cli::parse();

    match args.command.as_str() {
        "interactive" => {
            if let Err(e) = run_interactive_mode() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            return;
        }
        "tui" => {
            if let Err(e) = run_tui_mode() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            return;
        }
        _ => {}
    }

    let mut todo = TodoList::load(TODO_FILE);

    let status_filter = args.status.as_deref().and_then(|s| match s {
        "todo" => Some(Status::Todo),
        "done" => Some(Status::Done),
        _ => None,
    });

    let priority_value = args.priority.as_deref().and_then(|p| match p {
        "low" => Some(Priority::Low),
        "medium" => Some(Priority::Medium),
        "high" => Some(Priority::High),
        _ => None,
    });

    match handle_command(
        &mut todo,
        args.command.as_str(),
        args.key,
        status_filter,
        priority_value,
        args.due,
        args.tag,
        TODO_FILE,
    ) {
        Err(e) => println!("Error: {}", e),
        Ok(outcome) => println!("{}", format_outcome(outcome)),
    }
}
