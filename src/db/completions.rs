use std::collections::HashMap;

use jiff::Timestamp;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::executors::ExecutorId;
use super::projects::ProjectId;
use super::{from_millis, to_millis};
use crate::domain::TaskId;

#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    pub id: Uuid,
    pub task_id: TaskId,
    pub executor_id: ExecutorId,
    pub completed_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Upsert {
    Created,
    Updated,
    /// The completion already exists with exactly these values.
    Unchanged,
    UnknownTask,
    UnknownExecutor,
}

/// Creates or replaces a completion. Repeating the same call is a no-op, so clients
/// can safely retry.
pub async fn upsert(pool: &SqlitePool, completion: &Completion) -> anyhow::Result<Upsert> {
    let id = completion.id.to_string();
    let completed_at = to_millis(completion.completed_at);
    // IMMEDIATE takes the write lock up front, so the checks below cannot race with
    // another writer (a deferred transaction would fail to upgrade with SQLITE_BUSY).
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let existing = sqlx::query!(
        "SELECT task_id, executor_id, completed_at FROM completions WHERE id = ?",
        id
    )
    .fetch_optional(&mut *tx)
    .await?;
    // Checked before the task, so a retry still succeeds after the task was archived.
    if let Some(existing) = &existing
        && existing.task_id == completion.task_id
        && existing.executor_id == completion.executor_id
        && existing.completed_at == completed_at
    {
        return Ok(Upsert::Unchanged);
    }

    let task_is_active = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM tasks WHERE id = ? AND archived_at IS NULL) AS "active!: bool""#,
        completion.task_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !task_is_active {
        return Ok(Upsert::UnknownTask);
    }
    let executor_exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM executors WHERE id = ?) AS "exists!: bool""#,
        completion.executor_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !executor_exists {
        return Ok(Upsert::UnknownExecutor);
    }

    sqlx::query!(
        "INSERT INTO completions (id, task_id, executor_id, completed_at) VALUES (?, ?, ?, ?)
         ON CONFLICT (id) DO UPDATE SET
             task_id = excluded.task_id,
             executor_id = excluded.executor_id,
             completed_at = excluded.completed_at",
        id,
        completion.task_id,
        completion.executor_id,
        completed_at
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(if existing.is_some() {
        Upsert::Updated
    } else {
        Upsert::Created
    })
}

/// Deleting a completion that does not exist is not an error, so undo is idempotent.
pub async fn delete(pool: &SqlitePool, id: Uuid) -> anyhow::Result<()> {
    let id = id.to_string();
    sqlx::query!("DELETE FROM completions WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}

/// The latest completion of every active task in a project.
pub async fn latest_per_task(
    pool: &SqlitePool,
    project_id: ProjectId,
) -> anyhow::Result<HashMap<TaskId, Completion>> {
    // SQLite takes the bare columns (id, executor_id) from the row that has the MAX.
    let rows = sqlx::query!(
        r#"SELECT c.id AS "id!", c.task_id AS "task_id!", c.executor_id AS "executor_id!",
               MAX(c.completed_at) AS "completed_at!: i64"
           FROM completions c
           JOIN tasks t ON t.id = c.task_id
           WHERE t.project_id = ? AND t.archived_at IS NULL
           GROUP BY c.task_id"#,
        project_id
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let completion = Completion {
                id: row.id.parse()?,
                task_id: row.task_id,
                executor_id: row.executor_id,
                completed_at: from_millis(row.completed_at)?,
            };
            Ok((completion.task_id, completion))
        })
        .collect()
}
