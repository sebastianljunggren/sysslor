use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, State};
use jiff::Timestamp;

use super::types::{Project, ProjectInput, ProjectView, TaskView};
use super::{ApiError, ApiResult, AppState, validate_name};
use crate::db;
use crate::db::projects::ProjectId;
use crate::domain::{sort_groups, sort_tasks};

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<ProjectId>,
) -> ApiResult<Json<ProjectView>> {
    let Some(project) = db::projects::get(&state.pool, id).await? else {
        return Err(ApiError::NotFound("no such project"));
    };
    let mut groups = db::groups::list(&state.pool, id).await?;
    sort_groups(&mut groups);
    let tasks = db::tasks::list_active(&state.pool, id).await?;
    let mut latest = db::completions::latest_per_task(&state.pool, id).await?;
    let last_completed: HashMap<_, _> = latest
        .iter()
        .map(|(&task_id, completion)| (task_id, completion.completed_at))
        .collect();

    let tasks = sort_tasks(tasks, &last_completed, Timestamp::now(), &state.family_tz)
        .into_iter()
        .map(|scheduled| {
            let last = latest.remove(&scheduled.task.id);
            TaskView::new(scheduled, last)
        })
        .collect();
    Ok(Json(ProjectView {
        project: project.into(),
        groups: groups.into_iter().map(Into::into).collect(),
        tasks,
    }))
}

pub async fn rename(
    State(state): State<AppState>,
    Path(id): Path<ProjectId>,
    Json(input): Json<ProjectInput>,
) -> ApiResult<Json<Project>> {
    let name = validate_name(&input.name)?;
    let Some(project) = db::projects::rename(&state.pool, id, &name).await? else {
        return Err(ApiError::NotFound("no such project"));
    };
    state.events.changed(Some(id));
    Ok(Json(project.into()))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;
    use sqlx::SqlitePool;

    use crate::api::test_util::*;
    use crate::api::types::Project;

    #[sqlx::test]
    async fn unknown_project_is_not_found(pool: SqlitePool) {
        let (status, _) = request(&pool, "GET", "/api/projects/99", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn default_project_starts_empty(pool: SqlitePool) {
        let view = default_project(&pool).await;
        assert_eq!(view.project.id, DEFAULT_PROJECT);
        assert_eq!(view.project.name, "Default");
        assert!(view.tasks.is_empty());
    }

    #[sqlx::test]
    async fn tasks_are_sorted_with_schedule(pool: SqlitePool) {
        let executor = create_executor(&pool, "Alice").await;
        let done = create_task(&pool, "Done", 7, 5).await;
        let low = create_task(&pool, "Low", 7, 1).await;
        let high = create_task(&pool, "High", 7, 4).await;
        let stale = create_task(&pool, "Stale", 1, 0).await;
        let completion = "0199b4a2-7c00-7000-8000-000000000001";
        complete(&pool, completion, done, executor, now_ms()).await;
        complete(
            &pool,
            "0199b4a2-7c00-7000-8000-000000000002",
            stale,
            executor,
            days_ago(3),
        )
        .await;

        let view = default_project(&pool).await;
        let ids: Vec<_> = view.tasks.iter().map(|t| t.task.id).collect();
        // Overdue by priority (never completed counts as overdue), then the rest.
        assert_eq!(ids, [high, low, stale, done]);

        let never = &view.tasks[0];
        assert!(never.overdue);
        assert_eq!((never.due.as_ref(), never.urgency), (None, None));
        assert_eq!(never.last_completion, None);

        let stale = &view.tasks[2];
        assert!(stale.overdue);
        assert_eq!(stale.urgency, Some(3.0));

        let done = &view.tasks[3];
        assert!(!done.overdue);
        assert_eq!(done.urgency, Some(0.0));
        assert!(done.due.is_some());
        let last = done.last_completion.as_ref().unwrap();
        assert_eq!((last.id.as_str(), last.executor_id), (completion, executor));
    }

    #[sqlx::test]
    async fn last_completion_is_the_latest_one(pool: SqlitePool) {
        let executor = create_executor(&pool, "Alice").await;
        let other = create_executor(&pool, "Bob").await;
        let task = create_task(&pool, "Vacuum", 7, 0).await;
        let latest = "0199b4a2-7c00-7000-8000-000000000002";
        let yesterday = days_ago(1);
        // Registered out of order, as a backdated completion would be.
        complete(&pool, latest, task, other, yesterday).await;
        complete(
            &pool,
            "0199b4a2-7c00-7000-8000-000000000003",
            task,
            executor,
            days_ago(2),
        )
        .await;

        let view = default_project(&pool).await;
        let last = view.tasks[0].last_completion.as_ref().unwrap();
        assert_eq!(last.id, latest);
        assert_eq!(last.executor_id, other);
        assert_eq!(last.completed_at, yesterday);
    }

    #[sqlx::test]
    async fn archived_tasks_are_hidden(pool: SqlitePool) {
        let kept = create_task(&pool, "Kept", 7, 0).await;
        let archived = create_task(&pool, "Archived", 7, 0).await;
        let (status, _) = request(&pool, "DELETE", &format!("/api/tasks/{archived}"), None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        let view = default_project(&pool).await;
        let ids: Vec<_> = view.tasks.iter().map(|t| t.task.id).collect();
        assert_eq!(ids, [kept]);
    }

    #[sqlx::test]
    async fn rename(pool: SqlitePool) {
        let (status, project): (_, Project) = json(
            &pool,
            "PUT",
            "/api/projects/1",
            Some(json!({ "name": " Home " })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(project.name, "Home");
        assert_eq!(default_project(&pool).await.project.name, "Home");

        let (status, _) = request(
            &pool,
            "PUT",
            "/api/projects/99",
            Some(json!({ "name": "X" })),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = request(
            &pool,
            "PUT",
            "/api/projects/1",
            Some(json!({ "name": " " })),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}
