use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use jiff::Timestamp;
use uuid::{Uuid, Version};

use super::types::{Completion, CompletionInput};
use super::{ApiError, ApiResult, AppState};
use crate::db;
use crate::db::completions::Upsert;

/// Creates or replaces a completion. The client picks the id, so retrying a request
/// (e.g. after a dropped connection) never registers the task twice. Replacing is how
/// a completion is backdated.
pub async fn put(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CompletionInput>,
) -> ApiResult<(StatusCode, Json<Completion>)> {
    let id = parse_id(&id)?;
    if id.get_version() != Some(Version::SortRand) {
        return Err(ApiError::Invalid(format!(
            "completion id {id} is not a UUIDv7"
        )));
    }
    let completed_at = Timestamp::from_millisecond(input.completed_at).map_err(|_| {
        ApiError::Invalid(format!(
            "completed_at {} is out of range",
            input.completed_at
        ))
    })?;
    let completion = db::completions::Completion {
        id,
        task_id: input.task_id,
        executor_id: input.executor_id,
        completed_at,
    };

    let status = match db::completions::upsert(&state.pool, &completion).await? {
        Upsert::Created => StatusCode::CREATED,
        Upsert::Updated | Upsert::Unchanged => StatusCode::OK,
        Upsert::UnknownTask => {
            return Err(ApiError::Invalid(format!(
                "task {} does not exist or is archived",
                input.task_id
            )));
        }
        Upsert::UnknownExecutor => {
            return Err(ApiError::Invalid(format!(
                "executor {} does not exist",
                input.executor_id
            )));
        }
    };
    Ok((status, Json(completion.into())))
}

/// Undoes a completion. Succeeds even if it is already gone.
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let id = parse_id(&id)?;
    db::completions::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn parse_id(id: &str) -> ApiResult<Uuid> {
    Uuid::parse_str(id)
        .map_err(|_| ApiError::Invalid(format!("completion id {id:?} is not a UUID")))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use sqlx::SqlitePool;

    use crate::api::test_util::*;
    use crate::api::types::Completion;

    const ID: &str = "0199b4a2-7c00-7000-8000-000000000001";

    async fn setup(pool: &SqlitePool) -> (i64, i64) {
        let task = create_task(pool, "Vacuum", 7, 0).await;
        let executor = create_executor(pool, "Alice").await;
        (task, executor)
    }

    #[sqlx::test]
    async fn put_is_idempotent(pool: SqlitePool) {
        let (task, executor) = setup(&pool).await;
        let at = now_ms();

        let (status, first) = complete(&pool, ID, task, executor, at).await;
        assert_eq!(status, StatusCode::CREATED);
        let (status, second) = complete(&pool, ID, task, executor, at).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(first, second);

        let completion: Completion = serde_json::from_str(&first).unwrap();
        assert_eq!(
            completion,
            Completion {
                id: ID.into(),
                task_id: task,
                executor_id: executor,
                completed_at: at,
            }
        );
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM completions")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn id_is_normalized(pool: SqlitePool) {
        let (task, executor) = setup(&pool).await;
        let at = now_ms();
        complete(&pool, ID, task, executor, at).await;
        let (status, body) = complete(&pool, &ID.to_uppercase(), task, executor, at).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains(ID));
    }

    #[sqlx::test]
    async fn put_backdates_an_existing_completion(pool: SqlitePool) {
        let (task, executor) = setup(&pool).await;
        let now = now_ms();
        complete(&pool, ID, task, executor, now).await;

        let yesterday = days_ago(1);
        let (status, _) = complete(&pool, ID, task, executor, yesterday).await;
        assert_eq!(status, StatusCode::OK);

        let view = default_project(&pool).await;
        let last = view.tasks[0].last_completion.as_ref().unwrap();
        assert_eq!(last.completed_at, yesterday);
        assert_eq!(view.tasks[0].urgency, Some(1.0 / 7.0));
    }

    #[sqlx::test]
    async fn invalid_ids_are_rejected(pool: SqlitePool) {
        let (task, executor) = setup(&pool).await;
        for id in ["not-a-uuid", "0199b4a2-7c00-4000-8000-000000000001"] {
            let (status, _) = complete(&pool, id, task, executor, now_ms()).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{id}");
        }
    }

    #[sqlx::test]
    async fn unknown_references_are_rejected(pool: SqlitePool) {
        let (task, executor) = setup(&pool).await;
        let (status, body) = complete(&pool, ID, 99, executor, now_ms()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("task 99"));
        let (status, body) = complete(&pool, ID, task, 99, now_ms()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("executor 99"));
        let (status, _) = complete(&pool, ID, task, executor, i64::MAX).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test]
    async fn archived_task_rejects_new_completions_but_accepts_retries(pool: SqlitePool) {
        let (task, executor) = setup(&pool).await;
        let at = now_ms();
        complete(&pool, ID, task, executor, at).await;
        request(&pool, "DELETE", &format!("/api/tasks/{task}"), None).await;

        let (status, _) = complete(&pool, ID, task, executor, at).await;
        assert_eq!(status, StatusCode::OK);
        let (status, _) = complete(
            &pool,
            "0199b4a2-7c00-7000-8000-000000000002",
            task,
            executor,
            at,
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test]
    async fn delete_undoes_and_is_idempotent(pool: SqlitePool) {
        let (task, executor) = setup(&pool).await;
        complete(&pool, ID, task, executor, now_ms()).await;
        let uri = format!("/api/completions/{ID}");

        for _ in 0..2 {
            let (status, _) = request(&pool, "DELETE", &uri, None).await;
            assert_eq!(status, StatusCode::NO_CONTENT);
        }
        let view = default_project(&pool).await;
        assert_eq!(view.tasks[0].last_completion, None);
        assert!(view.tasks[0].overdue);
    }
}
