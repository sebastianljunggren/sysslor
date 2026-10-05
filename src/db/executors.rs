use sqlx::SqlitePool;

pub type ExecutorId = i64;

#[derive(Debug, Clone, PartialEq)]
pub struct Executor {
    pub id: ExecutorId,
    pub name: String,
}

pub async fn list(pool: &SqlitePool) -> anyhow::Result<Vec<Executor>> {
    let executors = sqlx::query_as!(Executor, "SELECT id, name FROM executors ORDER BY name, id")
        .fetch_all(pool)
        .await?;
    Ok(executors)
}

pub async fn create(pool: &SqlitePool, name: &str) -> anyhow::Result<Executor> {
    let id = sqlx::query!("INSERT INTO executors (name) VALUES (?)", name)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(Executor {
        id,
        name: name.to_owned(),
    })
}

/// Returns `None` if there is no such executor.
pub async fn rename(
    pool: &SqlitePool,
    id: ExecutorId,
    name: &str,
) -> anyhow::Result<Option<Executor>> {
    let updated = sqlx::query!("UPDATE executors SET name = ? WHERE id = ?", name, id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok((updated > 0).then(|| Executor {
        id,
        name: name.to_owned(),
    }))
}
