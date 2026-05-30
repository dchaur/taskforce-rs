use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Status {
    Todo,
    Done,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "Low"),
            Priority::Medium => write!(f, "Medium"),
            Priority::High => write!(f, "High"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]

pub(crate) enum UndoAction {
    Add {
        key: String,
    },
    Edit {
        key: String,
        new_key: String,
        previous_item: TodoItem,
    },
    MarkDone {
        key: String,
        previous_item: TodoItem,
    },
    MarkTodo {
        key: String,
        previous_item: TodoItem,
    },
    Delete {
        key: String,
        previous_item: TodoItem,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TodoItem {
    pub name: String,
    pub status: Status,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub tags: HashSet<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ListInfo {
    pub name: String,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub tags: HashSet<String>,
}
