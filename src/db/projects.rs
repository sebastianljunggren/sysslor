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
