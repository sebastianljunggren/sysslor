use sqlx::SqlitePool;

pub type ProjectId = i64;

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
}

pub async fn get(pool: &SqlitePool, id: ProjectId) -> anyhow::Result<Option<Project>> {
    let project = sqlx::query_as!(Project, "SELECT id, name FROM projects WHERE id = ?", id)
        .fetch_optional(pool)
        .await?;
    Ok(project)
}

/// Returns `None` if there is no such project.
pub async fn rename(
    pool: &SqlitePool,
    id: ProjectId,
    name: &str,
) -> anyhow::Result<Option<Project>> {
    let updated = sqlx::query!("UPDATE projects SET name = ? WHERE id = ?", name, id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok((updated > 0).then(|| Project {
        id,
        name: name.to_owned(),
    }))
}
