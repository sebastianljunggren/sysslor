use anyhow::{Context, bail};
use jiff::Timestamp;
use sqlx::SqlitePool;

use super::projects::ProjectId;
use super::to_millis;
use crate::domain::{Cadence, CadenceUnit, Task, TaskId};

/// The user-editable fields of a task.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskFields {
    pub name: String,
    pub cadence: Cadence,
    pub priority: u8,
}

/// Lists the tasks in a project that are not archived, in no particular order.
pub async fn list_active(pool: &SqlitePool, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
    let rows = sqlx::query!(
        "SELECT id, name, cadence_amount, cadence_unit, priority
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
            )
        })
        .collect()
}

/// Returns `None` if there is no such project.
pub async fn create(
    pool: &SqlitePool,
    project_id: ProjectId,
    fields: &TaskFields,
    now: Timestamp,
) -> anyhow::Result<Option<Task>> {
    let amount = i64::from(fields.cadence.amount);
    let unit = unit_to_str(fields.cadence.unit);
    let created_at = to_millis(now);
    // Inserting via SELECT turns an unknown project into zero rows instead of a
    // foreign key error.
    let id = sqlx::query_scalar!(
        r#"INSERT INTO tasks (project_id, name, cadence_amount, cadence_unit, priority, created_at)
         SELECT id, ?, ?, ?, ?, ? FROM projects WHERE id = ?
         RETURNING id AS "id!""#,
        fields.name,
        amount,
        unit,
        fields.priority,
        created_at,
        project_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(id.map(|id| Task {
        id,
        name: fields.name.clone(),
        cadence: fields.cadence,
        priority: fields.priority,
    }))
}

/// Returns `None` if there is no such task or it is archived.
pub async fn update(
    pool: &SqlitePool,
    id: TaskId,
    fields: &TaskFields,
) -> anyhow::Result<Option<Task>> {
    let amount = i64::from(fields.cadence.amount);
    let unit = unit_to_str(fields.cadence.unit);
    let updated = sqlx::query!(
        "UPDATE tasks SET name = ?, cadence_amount = ?, cadence_unit = ?, priority = ?
         WHERE id = ? AND archived_at IS NULL",
        fields.name,
        amount,
        unit,
        fields.priority,
        id
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok((updated > 0).then(|| Task {
        id,
        name: fields.name.clone(),
        cadence: fields.cadence,
        priority: fields.priority,
    }))
}

/// Soft-deletes a task so its completion history survives. Returns `false` if there
/// is no such task or it is already archived.
pub async fn archive(pool: &SqlitePool, id: TaskId, now: Timestamp) -> anyhow::Result<bool> {
    let archived_at = to_millis(now);
    let updated = sqlx::query!(
        "UPDATE tasks SET archived_at = ? WHERE id = ? AND archived_at IS NULL",
        archived_at,
        id
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(updated > 0)
}

fn task(id: TaskId, name: String, amount: i64, unit: &str, priority: i64) -> anyhow::Result<Task> {
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
    })
}

fn unit_to_str(unit: CadenceUnit) -> &'static str {
    match unit {
        CadenceUnit::Days => "days",
        CadenceUnit::Weeks => "weeks",
    }
}
