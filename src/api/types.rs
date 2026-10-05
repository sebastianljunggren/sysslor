//! The JSON contract. `cargo test` exports these as TypeScript to `web/src/lib/api/`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db;
use crate::domain::{self, Schedule, ScheduledTask};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CadenceUnit {
    Days,
    Weeks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Cadence {
    pub amount: u32,
    pub unit: CadenceUnit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

/// Body for renaming a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectInput {
    pub name: String,
}

/// A project with its groups, sorted by name, and its tasks, sorted so the ones needing
/// attention come first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectView {
    pub project: Project,
    pub groups: Vec<Group>,
    pub tasks: Vec<TaskView>,
}

/// A Solarized accent, named like the palette variables in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GroupColor {
    Yellow,
    Orange,
    Red,
    Magenta,
    Violet,
    Blue,
    Cyan,
    Green,
}

/// An optional grouping of tasks within a project, such as a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Group {
    pub id: i64,
    pub name: String,
    /// `null` if the group has no color.
    pub color: Option<GroupColor>,
}

/// Body for creating or updating a group. Updates replace both fields, so an omitted
/// `color` clears it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GroupInput {
    pub name: String,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub color: Option<GroupColor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub cadence: Cadence,
    /// 0-5, where 5 is the highest priority.
    pub priority: u8,
    /// `null` if the task is not in a group.
    pub group_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskView {
    #[serde(flatten)]
    pub task: Task,
    /// `null` if the task has never been completed.
    pub last_completion: Option<Completion>,
    /// The calendar day (`YYYY-MM-DD`, configured time zone) the task is due again.
    /// `null` if the task has never been completed, which means it is due now.
    pub due: Option<String>,
    /// Days since the last completion divided by the cadence in days.
    /// `null` if the task has never been completed.
    pub urgency: Option<f64>,
    /// Due today or earlier (`urgency >= 1`, or never completed).
    pub overdue: bool,
}

/// Body for creating or updating a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskInput {
    pub name: String,
    pub cadence: Cadence,
    pub priority: u8,
    /// A group in the task's project, or `null` for none.
    pub group_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Executor {
    pub id: i64,
    pub name: String,
}

/// Body for creating or renaming an executor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecutorInput {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Completion {
    /// Client-generated UUIDv7.
    pub id: String,
    pub task_id: i64,
    pub executor_id: i64,
    /// UTC unix milliseconds.
    pub completed_at: i64,
}

/// Body for `PUT /api/completions/{id}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompletionInput {
    pub task_id: i64,
    pub executor_id: i64,
    /// UTC unix milliseconds.
    pub completed_at: i64,
}

/// Sent on `GET /api/events` (as SSE `data`). Events carry no payload: clients refetch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum Event {
    /// `project` is `null` when global data (executors) changed or events were
    /// missed; refetch everything then.
    Changed { project: Option<i64> },
}

/// Body for `POST /api/login`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoginInput {
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ErrorBody {
    pub error: String,
}

impl From<domain::CadenceUnit> for CadenceUnit {
    fn from(unit: domain::CadenceUnit) -> Self {
        match unit {
            domain::CadenceUnit::Days => Self::Days,
            domain::CadenceUnit::Weeks => Self::Weeks,
        }
    }
}

impl From<CadenceUnit> for domain::CadenceUnit {
    fn from(unit: CadenceUnit) -> Self {
        match unit {
            CadenceUnit::Days => Self::Days,
            CadenceUnit::Weeks => Self::Weeks,
        }
    }
}

impl From<domain::Cadence> for Cadence {
    fn from(cadence: domain::Cadence) -> Self {
        Self {
            amount: cadence.amount,
            unit: cadence.unit.into(),
        }
    }
}

impl From<Cadence> for domain::Cadence {
    fn from(cadence: Cadence) -> Self {
        Self {
            amount: cadence.amount,
            unit: cadence.unit.into(),
        }
    }
}

impl From<db::projects::Project> for Project {
    fn from(project: db::projects::Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
        }
    }
}

impl From<domain::Task> for Task {
    fn from(task: domain::Task) -> Self {
        Self {
            id: task.id,
            name: task.name,
            cadence: task.cadence.into(),
            priority: task.priority,
            group_id: task.group_id,
        }
    }
}

impl From<domain::GroupColor> for GroupColor {
    fn from(color: domain::GroupColor) -> Self {
        match color {
            domain::GroupColor::Yellow => Self::Yellow,
            domain::GroupColor::Orange => Self::Orange,
            domain::GroupColor::Red => Self::Red,
            domain::GroupColor::Magenta => Self::Magenta,
            domain::GroupColor::Violet => Self::Violet,
            domain::GroupColor::Blue => Self::Blue,
            domain::GroupColor::Cyan => Self::Cyan,
            domain::GroupColor::Green => Self::Green,
        }
    }
}

impl From<GroupColor> for domain::GroupColor {
    fn from(color: GroupColor) -> Self {
        match color {
            GroupColor::Yellow => Self::Yellow,
            GroupColor::Orange => Self::Orange,
            GroupColor::Red => Self::Red,
            GroupColor::Magenta => Self::Magenta,
            GroupColor::Violet => Self::Violet,
            GroupColor::Blue => Self::Blue,
            GroupColor::Cyan => Self::Cyan,
            GroupColor::Green => Self::Green,
        }
    }
}

impl From<domain::Group> for Group {
    fn from(group: domain::Group) -> Self {
        Self {
            id: group.id,
            name: group.name,
            color: group.color.map(Into::into),
        }
    }
}

impl From<db::executors::Executor> for Executor {
    fn from(executor: db::executors::Executor) -> Self {
        Self {
            id: executor.id,
            name: executor.name,
        }
    }
}

impl From<db::completions::Completion> for Completion {
    fn from(completion: db::completions::Completion) -> Self {
        Self {
            id: completion.id.to_string(),
            task_id: completion.task_id,
            executor_id: completion.executor_id,
            completed_at: completion.completed_at.as_millisecond(),
        }
    }
}

impl TaskView {
    pub fn new(
        scheduled: ScheduledTask,
        last_completion: Option<db::completions::Completion>,
    ) -> Self {
        let (due, urgency) = match scheduled.schedule {
            Schedule::NeverCompleted => (None, None),
            Schedule::Completed { due, urgency, .. } => (Some(due.to_string()), Some(urgency)),
        };
        Self {
            overdue: scheduled.schedule.is_overdue(),
            task: scheduled.task.into(),
            last_completion: last_completion.map(Into::into),
            due,
            urgency,
        }
    }
}
