use anyhow::{Context, bail};
use jiff::Timestamp;
use sqlx::{SqliteConnection, SqlitePool};

use super::projects::ProjectId;
use super::to_millis;
use crate::domain::{Cadence, CadenceUnit, GroupId, Task, TaskId};

/// The user-editable fields of a task.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskFields {
    pub name: String,
    pub cadence: Cadence,
    pub priority: u8,
    pub group_id: Option<GroupId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Write {
    /// Carries the written task and the project it belongs to.
    Done(ProjectId, Task),
    /// There is no such project (create) or no such active task (update).
    NotFound,
    /// The group does not exist in the task's project.
    UnknownGroup,
}

/// Lists the tasks in a project that are not archived, in no particular order.
pub async fn list_active(pool: &SqlitePool, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
    let rows = sqlx::query!(
        "SELECT id, name, cadence_amount, cadence_unit, priority, group_id
         FROM tasks
         WHERE project_id = ? AND archived_at IS NULL",
        project_id
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            task(
                row.id,
                row.name,
                row.cadence_amount,
                &row.cadence_unit,
                row.priority,
                row.group_id,
            )
        })
        .collect()
}

pub async fn create(
    pool: &SqlitePool,
    project_id: ProjectId,
    fields: &TaskFields,
    now: Timestamp,
) -> anyhow::Result<Write> {
    let amount = i64::from(fields.cadence.amount);
    let unit = unit_to_str(fields.cadence.unit);
    let created_at = to_millis(now);
    // IMMEDIATE so the group cannot be deleted between the check and the insert.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let project_exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM projects WHERE id = ?) AS "exists!: bool""#,
        project_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !project_exists {
        return Ok(Write::NotFound);
    }
    if !group_in_project(&mut tx, fields.group_id, project_id).await? {
        return Ok(Write::UnknownGroup);
    }

    let id = sqlx::query_scalar!(
        r#"INSERT INTO tasks
             (project_id, name, cadence_amount, cadence_unit, priority, group_id, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         RETURNING id AS "id!""#,
        project_id,
        fields.name,
        amount,
        unit,
        fields.priority,
        fields.group_id,
        created_at
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Write::Done(project_id, with_fields(id, fields)))
}

pub async fn update(pool: &SqlitePool, id: TaskId, fields: &TaskFields) -> anyhow::Result<Write> {
    let amount = i64::from(fields.cadence.amount);
    let unit = unit_to_str(fields.cadence.unit);
    // IMMEDIATE so the group cannot be deleted between the check and the update.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let project_id = sqlx::query_scalar!(
        "SELECT project_id FROM tasks WHERE id = ? AND archived_at IS NULL",
        id
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(project_id) = project_id else {
        return Ok(Write::NotFound);
    };
    if !group_in_project(&mut tx, fields.group_id, project_id).await? {
        return Ok(Write::UnknownGroup);
    }

    sqlx::query!(
        "UPDATE tasks SET name = ?, cadence_amount = ?, cadence_unit = ?, priority = ?, group_id = ?
         WHERE id = ?",
        fields.name,
        amount,
        unit,
        fields.priority,
        fields.group_id,
        id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Write::Done(project_id, with_fields(id, fields)))
}

/// No group always counts as valid.
async fn group_in_project(
    tx: &mut SqliteConnection,
    group_id: Option<GroupId>,
    project_id: ProjectId,
) -> anyhow::Result<bool> {
    let Some(group_id) = group_id else {
        return Ok(true);
    };
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (
             SELECT 1 FROM task_groups WHERE id = ? AND project_id = ?
         ) AS "exists!: bool""#,
        group_id,
        project_id
    )
    .fetch_one(tx)
    .await?;
    Ok(exists)
}

fn with_fields(id: TaskId, fields: &TaskFields) -> Task {
    Task {
        id,
        name: fields.name.clone(),
        cadence: fields.cadence,
        priority: fields.priority,
        group_id: fields.group_id,
    }
}

/// Soft-deletes a task so its completion history survives. Returns the project the
/// task belongs to, or `None` if there is no such task or it is already archived.
pub async fn archive(
    pool: &SqlitePool,
    id: TaskId,
    now: Timestamp,
) -> anyhow::Result<Option<ProjectId>> {
    let archived_at = to_millis(now);
    let project_id = sqlx::query_scalar!(
        "UPDATE tasks SET archived_at = ? WHERE id = ? AND archived_at IS NULL
         RETURNING project_id",
        archived_at,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(project_id)
}

fn task(
    id: TaskId,
    name: String,
    amount: i64,
    unit: &str,
    priority: i64,
    group_id: Option<GroupId>,
) -> anyhow::Result<Task> {
    let amount = u32::try_from(amount)
        .with_context(|| format!("task {id} has invalid cadence amount {amount}"))?;
    let unit = match unit {
        "days" => CadenceUnit::Days,
        "weeks" => CadenceUnit::Weeks,
        other => bail!("task {id} has invalid cadence unit {other:?}"),
    };
    let priority = u8::try_from(priority)
        .with_context(|| format!("task {id} has invalid priority {priority}"))?;
    Ok(Task {
        id,
        name,
        cadence: Cadence { amount, unit },
        priority,
        group_id,
    })
}

fn unit_to_str(unit: CadenceUnit) -> &'static str {
    match unit {
        CadenceUnit::Days => "days",
        CadenceUnit::Weeks => "weeks",
    }
}
