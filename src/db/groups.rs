use sqlx::SqlitePool;

use super::projects::ProjectId;
use crate::domain::{Group, GroupId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delete {
    /// Carries the project the group belonged to.
    Deleted(ProjectId),
    NotFound,
    /// The group still has active tasks.
    NotEmpty,
}

/// Lists the groups in a project, in no particular order.
pub async fn list(pool: &SqlitePool, project_id: ProjectId) -> anyhow::Result<Vec<Group>> {
    let groups = sqlx::query_as!(
        Group,
        "SELECT id, name FROM task_groups WHERE project_id = ?",
        project_id
    )
    .fetch_all(pool)
    .await?;
    Ok(groups)
}

/// Returns `None` if there is no such project.
pub async fn create(
    pool: &SqlitePool,
    project_id: ProjectId,
    name: &str,
) -> anyhow::Result<Option<Group>> {
    // Inserting via SELECT turns an unknown project into zero rows instead of a
    // foreign key error.
    let id = sqlx::query_scalar!(
        r#"INSERT INTO task_groups (project_id, name)
         SELECT id, ? FROM projects WHERE id = ?
         RETURNING id AS "id!""#,
        name,
        project_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(id.map(|id| Group {
        id,
        name: name.to_owned(),
    }))
}

/// Returns the group and the project it belongs to, or `None` if there is no such group.
pub async fn rename(
    pool: &SqlitePool,
    id: GroupId,
    name: &str,
) -> anyhow::Result<Option<(ProjectId, Group)>> {
    let project_id = sqlx::query_scalar!(
        "UPDATE task_groups SET name = ? WHERE id = ? RETURNING project_id",
        name,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(project_id.map(|project_id| {
        let group = Group {
            id,
            name: name.to_owned(),
        };
        (project_id, group)
    }))
}

/// Deletes a group that has no active tasks. Archived tasks lose their group, since
/// they are never shown again.
pub async fn delete(pool: &SqlitePool, id: GroupId) -> anyhow::Result<Delete> {
    // IMMEDIATE so no task can be added to the group between the check and the delete.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let project_id = sqlx::query_scalar!("SELECT project_id FROM task_groups WHERE id = ?", id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(project_id) = project_id else {
        return Ok(Delete::NotFound);
    };
    let has_active_tasks = sqlx::query_scalar!(
        r#"SELECT EXISTS (
             SELECT 1 FROM tasks WHERE group_id = ? AND archived_at IS NULL
         ) AS "exists!: bool""#,
        id
    )
    .fetch_one(&mut *tx)
    .await?;
    if has_active_tasks {
        return Ok(Delete::NotEmpty);
    }

    sqlx::query!("UPDATE tasks SET group_id = NULL WHERE group_id = ?", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM task_groups WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Delete::Deleted(project_id))
}
