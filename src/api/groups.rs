use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use super::types::{Group, GroupInput};
use super::{ApiError, ApiResult, AppState, validate_name};
use crate::db;
use crate::db::groups::Delete;
use crate::db::projects::ProjectId;
use crate::domain::GroupId;

pub async fn create(
    State(state): State<AppState>,
    Path(project_id): Path<ProjectId>,
    Json(input): Json<GroupInput>,
) -> ApiResult<(StatusCode, Json<Group>)> {
    let name = validate_name(&input.name)?;
    let color = input.color.map(Into::into);
    let Some(group) = db::groups::create(&state.pool, project_id, &name, color).await? else {
        return Err(ApiError::NotFound("no such project"));
    };
    state.events.changed(Some(project_id));
    Ok((StatusCode::CREATED, Json(group.into())))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<GroupId>,
    Json(input): Json<GroupInput>,
) -> ApiResult<Json<Group>> {
    let name = validate_name(&input.name)?;
    let color = input.color.map(Into::into);
    let Some((project_id, group)) = db::groups::update(&state.pool, id, &name, color).await? else {
        return Err(ApiError::NotFound("no such group"));
    };
    state.events.changed(Some(project_id));
    Ok(Json(group.into()))
}

/// Only empty groups can be deleted, so no task silently loses its group.
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<GroupId>,
) -> ApiResult<StatusCode> {
    match db::groups::delete(&state.pool, id).await? {
        Delete::Deleted(project_id) => {
            state.events.changed(Some(project_id));
            Ok(StatusCode::NO_CONTENT)
        }
        Delete::NotFound => Err(ApiError::NotFound("no such group")),
        Delete::NotEmpty => Err(ApiError::Conflict("group has tasks")),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;
    use sqlx::SqlitePool;

    use crate::api::test_util::*;
    use crate::api::types::{Group, GroupColor};

    #[sqlx::test]
    async fn create_and_list_sorted_by_name(pool: SqlitePool) {
        let (status, group): (_, Group) = json(
            &pool,
            "POST",
            "/api/projects/1/groups",
            Some(json!({ "name": " kitchen " })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(group.name, "kitchen");
        assert_eq!(group.color, None);
        let bathroom = create_group(&pool, "Bathroom").await;

        let view = default_project(&pool).await;
        let ids: Vec<_> = view.groups.iter().map(|g| g.id).collect();
        assert_eq!(ids, [bathroom, group.id]);
    }

    #[sqlx::test]
    async fn create_rejects_unknown_project_and_bad_names(pool: SqlitePool) {
        let (status, _) = request(
            &pool,
            "POST",
            "/api/projects/99/groups",
            Some(json!({ "name": "Kitchen" })),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = request(
            &pool,
            "POST",
            "/api/projects/1/groups",
            Some(json!({ "name": " " })),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(default_project(&pool).await.groups.is_empty());
    }

    #[sqlx::test]
    async fn rename(pool: SqlitePool) {
        let id = create_group(&pool, "Kitchen").await;
        let (status, group): (_, Group) = json(
            &pool,
            "PUT",
            &format!("/api/groups/{id}"),
            Some(json!({ "name": "Pantry" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            group,
            Group {
                id,
                name: "Pantry".to_owned(),
                color: None,
            }
        );
        assert_eq!(default_project(&pool).await.groups, [group]);

        let (status, _) =
            request(&pool, "PUT", "/api/groups/99", Some(json!({ "name": "X" }))).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn create_with_color(pool: SqlitePool) {
        let (status, group): (_, Group) = json(
            &pool,
            "POST",
            "/api/projects/1/groups",
            Some(json!({ "name": "Kitchen", "color": "orange" })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(group.color, Some(GroupColor::Orange));
        assert_eq!(default_project(&pool).await.groups, [group]);
    }

    #[sqlx::test]
    async fn update_sets_and_clears_color(pool: SqlitePool) {
        let id = create_group(&pool, "Kitchen").await;
        let uri = format!("/api/groups/{id}");
        let (status, group): (_, Group) = json(
            &pool,
            "PUT",
            &uri,
            Some(json!({ "name": "Kitchen", "color": "cyan" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(group.color, Some(GroupColor::Cyan));
        assert_eq!(default_project(&pool).await.groups, [group]);

        let (status, group): (_, Group) = json(
            &pool,
            "PUT",
            &uri,
            Some(json!({ "name": "Kitchen", "color": null })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(group.color, None);
        assert_eq!(default_project(&pool).await.groups, [group]);
    }

    #[sqlx::test]
    async fn rejects_unknown_color(pool: SqlitePool) {
        let (status, _) = request(
            &pool,
            "POST",
            "/api/projects/1/groups",
            Some(json!({ "name": "Kitchen", "color": "pink" })),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(default_project(&pool).await.groups.is_empty());
    }

    #[sqlx::test]
    async fn delete_empty_group(pool: SqlitePool) {
        let id = create_group(&pool, "Kitchen").await;
        let uri = format!("/api/groups/{id}");
        let (status, _) = request(&pool, "DELETE", &uri, None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(default_project(&pool).await.groups.is_empty());

        let (status, _) = request(&pool, "DELETE", &uri, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn delete_group_with_active_task_conflicts(pool: SqlitePool) {
        let group = create_group(&pool, "Kitchen").await;
        create_task_in(&pool, "Wipe counters", Some(group)).await;

        let (status, _) = request(&pool, "DELETE", &format!("/api/groups/{group}"), None).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(default_project(&pool).await.groups.len(), 1);
    }

    #[sqlx::test]
    async fn delete_group_with_only_archived_tasks(pool: SqlitePool) {
        let group = create_group(&pool, "Kitchen").await;
        let task = create_task_in(&pool, "Wipe counters", Some(group)).await;
        request(&pool, "DELETE", &format!("/api/tasks/{task}"), None).await;

        let (status, _) = request(&pool, "DELETE", &format!("/api/groups/{group}"), None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let group_id: Option<i64> = sqlx::query_scalar("SELECT group_id FROM tasks WHERE id = ?")
            .bind(task)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(group_id, None);
    }
}
