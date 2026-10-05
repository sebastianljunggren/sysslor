use anyhow::bail;
use sqlx::SqlitePool;

use super::projects::ProjectId;
use crate::domain::{Group, GroupColor, GroupId};

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
    let rows = sqlx::query!(
        "SELECT id, name, color FROM task_groups WHERE project_id = ?",
        project_id
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let color = row
                .color
                .map(|color| color_from_str(row.id, &color))
                .transpose()?;
            Ok(Group {
                id: row.id,
                name: row.name,
                color,
            })
        })
        .collect()
}

/// Returns `None` if there is no such project.
pub async fn create(
    pool: &SqlitePool,
    project_id: ProjectId,
    name: &str,
    color: Option<GroupColor>,
) -> anyhow::Result<Option<Group>> {
    let color_str = color.map(color_to_str);
    // Inserting via SELECT turns an unknown project into zero rows instead of a
    // foreign key error.
    let id = sqlx::query_scalar!(
        r#"INSERT INTO task_groups (project_id, name, color)
         SELECT id, ?, ? FROM projects WHERE id = ?
         RETURNING id AS "id!""#,
        name,
        color_str,
        project_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(id.map(|id| Group {
        id,
        name: name.to_owned(),
        color,
    }))
}

/// Returns the group and the project it belongs to, or `None` if there is no such group.
pub async fn update(
    pool: &SqlitePool,
    id: GroupId,
    name: &str,
    color: Option<GroupColor>,
) -> anyhow::Result<Option<(ProjectId, Group)>> {
    let color_str = color.map(color_to_str);
    let project_id = sqlx::query_scalar!(
        "UPDATE task_groups SET name = ?, color = ? WHERE id = ? RETURNING project_id",
        name,
        color_str,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(project_id.map(|project_id| {
        let group = Group {
            id,
            name: name.to_owned(),
            color,
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

fn color_from_str(id: GroupId, color: &str) -> anyhow::Result<GroupColor> {
    Ok(match color {
        "yellow" => GroupColor::Yellow,
        "orange" => GroupColor::Orange,
        "red" => GroupColor::Red,
        "magenta" => GroupColor::Magenta,
        "violet" => GroupColor::Violet,
        "blue" => GroupColor::Blue,
        "cyan" => GroupColor::Cyan,
        "green" => GroupColor::Green,
        other => bail!("group {id} has invalid color {other:?}"),
    })
}

fn color_to_str(color: GroupColor) -> &'static str {
    match color {
        GroupColor::Yellow => "yellow",
        GroupColor::Orange => "orange",
        GroupColor::Red => "red",
        GroupColor::Magenta => "magenta",
        GroupColor::Violet => "violet",
        GroupColor::Blue => "blue",
        GroupColor::Cyan => "cyan",
        GroupColor::Green => "green",
    }
}
