use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use super::types::{Executor, ExecutorInput};
use super::{ApiError, ApiResult, AppState, validate_name};
use crate::db;
use crate::db::executors::ExecutorId;

pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<Executor>>> {
    let executors = db::executors::list(&state.pool).await?;
    Ok(Json(executors.into_iter().map(Into::into).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<ExecutorInput>,
) -> ApiResult<(StatusCode, Json<Executor>)> {
    let name = validate_name(&input.name)?;
    let executor = db::executors::create(&state.pool, &name).await?;
    // Executors are shared by all projects.
    state.events.changed(None);
    Ok((StatusCode::CREATED, Json(executor.into())))
}

pub async fn rename(
    State(state): State<AppState>,
    Path(id): Path<ExecutorId>,
    Json(input): Json<ExecutorInput>,
) -> ApiResult<Json<Executor>> {
    let name = validate_name(&input.name)?;
    let Some(executor) = db::executors::rename(&state.pool, id, &name).await? else {
        return Err(ApiError::NotFound("no such executor"));
    };
    state.events.changed(None);
    Ok(Json(executor.into()))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;
    use sqlx::SqlitePool;

    use crate::api::test_util::*;
    use crate::api::types::Executor;

    #[sqlx::test]
    async fn executors_are_listed_by_name(pool: SqlitePool) {
        let bob = create_executor(&pool, "Bob").await;
        let alice = create_executor(&pool, " Alice ").await;
        let (status, executors): (_, Vec<Executor>) =
            json(&pool, "GET", "/api/executors", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            executors,
            [
                Executor {
                    id: alice,
                    name: "Alice".into()
                },
                Executor {
                    id: bob,
                    name: "Bob".into()
                },
            ]
        );
    }

    #[sqlx::test]
    async fn rename(pool: SqlitePool) {
        let id = create_executor(&pool, "Bob").await;
        let (status, executor): (_, Executor) = json(
            &pool,
            "PUT",
            &format!("/api/executors/{id}"),
            Some(json!({ "name": "Robert" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(executor.name, "Robert");

        let (status, _) = request(
            &pool,
            "PUT",
            "/api/executors/99",
            Some(json!({ "name": "X" })),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn empty_name_is_rejected(pool: SqlitePool) {
        let (status, _) =
            request(&pool, "POST", "/api/executors", Some(json!({ "name": "" }))).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}
