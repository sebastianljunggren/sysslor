use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use jiff::Timestamp;

use super::types::{Task, TaskInput};
use super::{ApiError, ApiResult, AppState, validate_name};
use crate::db;
use crate::db::projects::ProjectId;
use crate::db::tasks::TaskFields;
use crate::domain::TaskId;

pub async fn create(
    State(state): State<AppState>,
    Path(project_id): Path<ProjectId>,
    Json(input): Json<TaskInput>,
) -> ApiResult<(StatusCode, Json<Task>)> {
    let fields = validate(input)?;
    let Some(task) = db::tasks::create(&state.pool, project_id, &fields, Timestamp::now()).await?
    else {
        return Err(ApiError::NotFound("no such project"));
    };
    state.events.changed(Some(project_id));
    Ok((StatusCode::CREATED, Json(task.into())))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<TaskId>,
    Json(input): Json<TaskInput>,
) -> ApiResult<Json<Task>> {
    let fields = validate(input)?;
    let Some((project_id, task)) = db::tasks::update(&state.pool, id, &fields).await? else {
        return Err(ApiError::NotFound("no such task"));
    };
    state.events.changed(Some(project_id));
    Ok(Json(task.into()))
}

/// Archives the task. Its completions are kept.
pub async fn archive(
    State(state): State<AppState>,
    Path(id): Path<TaskId>,
) -> ApiResult<StatusCode> {
    let Some(project_id) = db::tasks::archive(&state.pool, id, Timestamp::now()).await? else {
        return Err(ApiError::NotFound("no such task"));
    };
    state.events.changed(Some(project_id));
    Ok(StatusCode::NO_CONTENT)
}

fn validate(input: TaskInput) -> ApiResult<TaskFields> {
    if input.cadence.amount == 0 {
        return Err(ApiError::Invalid(
            "cadence amount must be at least 1".to_owned(),
        ));
    }
    if input.priority > 5 {
        return Err(ApiError::Invalid(
            "priority must be between 0 and 5".to_owned(),
        ));
    }
    Ok(TaskFields {
        name: validate_name(&input.name)?,
        cadence: input.cadence.into(),
        priority: input.priority,
    })
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::{Value, json};
    use sqlx::SqlitePool;

    use crate::api::test_util::*;
    use crate::api::types::{Cadence, CadenceUnit, Task};

    fn input(name: &str, amount: u32, priority: u8) -> Value {
        json!({ "name": name, "cadence": { "amount": amount, "unit": "weeks" }, "priority": priority })
    }

    #[sqlx::test]
    async fn create_returns_the_task(pool: SqlitePool) {
        let (status, task): (_, Task) = json(
            &pool,
            "POST",
            "/api/projects/1/tasks",
            Some(input("  Vacuum ", 2, 3)),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(task.name, "Vacuum");
        assert_eq!(
            task.cadence,
            Cadence {
                amount: 2,
                unit: CadenceUnit::Weeks
            }
        );
        assert_eq!(task.priority, 3);
        assert_eq!(default_project(&pool).await.tasks[0].task, task);
    }

    #[sqlx::test]
    async fn create_in_unknown_project_is_not_found(pool: SqlitePool) {
        let (status, _) = request(
            &pool,
            "POST",
            "/api/projects/99/tasks",
            Some(input("Vacuum", 1, 0)),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn invalid_input_is_rejected(pool: SqlitePool) {
        for body in [
            input(" ", 1, 0),
            input("Vacuum", 0, 0),
            input("Vacuum", 1, 6),
            input(&"x".repeat(201), 1, 0),
        ] {
            let (status, _) =
                request(&pool, "POST", "/api/projects/1/tasks", Some(body.clone())).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        }
        let bad_unit = json!({ "name": "Vacuum", "cadence": { "amount": 1, "unit": "months" }, "priority": 0 });
        let (status, _) = request(&pool, "POST", "/api/projects/1/tasks", Some(bad_unit)).await;
        assert!(status.is_client_error());
        assert!(default_project(&pool).await.tasks.is_empty());
    }

    #[sqlx::test]
    async fn update_replaces_fields(pool: SqlitePool) {
        let id = create_task(&pool, "Vacuum", 7, 0).await;
        let (status, task): (_, Task) = json(
            &pool,
            "PUT",
            &format!("/api/tasks/{id}"),
            Some(input("Mop", 1, 5)),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!((task.id, task.name.as_str(), task.priority), (id, "Mop", 5));
        assert_eq!(default_project(&pool).await.tasks[0].task, task);
    }

    #[sqlx::test]
    async fn archived_tasks_cannot_be_updated_or_archived_again(pool: SqlitePool) {
        let id = create_task(&pool, "Vacuum", 7, 0).await;
        let uri = format!("/api/tasks/{id}");
        let (status, _) = request(&pool, "DELETE", &uri, None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        let (status, _) = request(&pool, "DELETE", &uri, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = request(&pool, "PUT", &uri, Some(input("Mop", 1, 0))).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn archiving_keeps_the_row(pool: SqlitePool) {
        let id = create_task(&pool, "Vacuum", 7, 0).await;
        request(&pool, "DELETE", &format!("/api/tasks/{id}"), None).await;
        let archived: Option<i64> =
            sqlx::query_scalar("SELECT archived_at FROM tasks WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(archived.is_some());
    }
}
