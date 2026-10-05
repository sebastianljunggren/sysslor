use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::response::sse::{self, KeepAlive, Sse};
use futures_util::{Stream, StreamExt, stream};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, watch};

use super::AppState;
use super::types::Event;
use crate::db::projects::ProjectId;

/// Fans out change notifications to every connected `/api/events` client.
#[derive(Clone)]
pub struct Events {
    tx: broadcast::Sender<Event>,
    shutdown: Arc<watch::Sender<bool>>,
}

impl Events {
    pub fn new() -> Self {
        // Clients that fall this far behind get a "refetch everything" event instead.
        let (tx, _) = broadcast::channel(16);
        Self {
            tx,
            shutdown: Arc::new(watch::channel(false).0),
        }
    }

    /// Call after a committed write that affects `project`, or `None` for global data.
    pub fn changed(&self, project: Option<ProjectId>) {
        // Sending only fails when nobody is listening, which is fine.
        let _ = self.tx.send(Event::Changed { project });
    }

    /// Ends all event streams. Graceful shutdown waits for open connections, and SSE
    /// connections never close by themselves.
    pub fn shutdown(&self) {
        self.shutdown.send_replace(true);
    }

    fn subscribe(&self) -> impl Stream<Item = Event> + use<> {
        let events = stream::unfold(self.tx.subscribe(), |mut rx| async move {
            let event = match rx.recv().await {
                Ok(event) => event,
                // Missed events can't be replayed, so make the client refetch it all.
                Err(RecvError::Lagged(_)) => Event::Changed { project: None },
                Err(RecvError::Closed) => return None,
            };
            Some((event, rx))
        });
        let mut shutdown = self.shutdown.subscribe();
        events.take_until(async move {
            // Errors only if the sender is dropped, which also means shutting down.
            let _ = shutdown.wait_for(|&done| done).await;
        })
    }
}

/// Streams change notifications. Clients refetch on (re)connect, so events missed
/// while disconnected don't matter.
pub async fn stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<sse::Event, axum::Error>>> {
    let events = state
        .events
        .subscribe()
        .map(|event| sse::Event::default().json_data(event));
    Sse::new(events).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::http::{StatusCode, header};
    use futures_util::{FutureExt, Stream, StreamExt};
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use sqlx::SqlitePool;

    use crate::api::test_util::*;
    use crate::api::types::Event;

    /// The events that are already queued, without waiting for more.
    fn drain(events: &mut (impl Stream<Item = Event> + Unpin)) -> Vec<Event> {
        std::iter::from_fn(|| events.next().now_or_never().flatten()).collect()
    }

    fn changed(project: Option<i64>) -> Event {
        Event::Changed { project }
    }

    #[sqlx::test]
    async fn committed_writes_send_changed(pool: SqlitePool) {
        let state = state(&pool);
        let mut events = Box::pin(state.events.subscribe());
        let task_input =
            json!({ "name": "Vacuum", "cadence": { "amount": 7, "unit": "days" }, "priority": 0 });
        let executor_input = json!({ "name": "Alice" });

        let (_, task) = request_with(
            &state,
            "POST",
            "/api/projects/1/tasks",
            Some(task_input.clone()),
        )
        .await;
        let task = serde_json::from_str::<Value>(&task).unwrap()["id"]
            .as_i64()
            .unwrap();
        let task_uri = format!("/api/tasks/{task}");
        request_with(&state, "PUT", &task_uri, Some(task_input.clone())).await;
        assert_eq!(drain(&mut events), [changed(Some(1)), changed(Some(1))]);

        let (_, executor) = request_with(
            &state,
            "POST",
            "/api/executors",
            Some(executor_input.clone()),
        )
        .await;
        let executor = serde_json::from_str::<Value>(&executor).unwrap()["id"]
            .as_i64()
            .unwrap();
        request_with(
            &state,
            "PUT",
            &format!("/api/executors/{executor}"),
            Some(executor_input),
        )
        .await;
        assert_eq!(drain(&mut events), [changed(None), changed(None)]);

        let completion_uri = "/api/completions/0199b4a2-7c00-7000-8000-000000000001";
        let completion = |completed_at| json!({ "task_id": task, "executor_id": executor, "completed_at": completed_at });
        let at = now_ms();
        request_with(&state, "PUT", completion_uri, Some(completion(at))).await;
        request_with(&state, "PUT", completion_uri, Some(completion(at))).await;
        request_with(&state, "PUT", completion_uri, Some(completion(days_ago(1)))).await;
        request_with(&state, "DELETE", completion_uri, None).await;
        request_with(&state, "DELETE", completion_uri, None).await;
        // Retries and no-op deletes change nothing, so they send nothing.
        assert_eq!(drain(&mut events), vec![changed(Some(1)); 3]);

        let (status, _) = request_with(&state, "DELETE", &task_uri, None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(drain(&mut events), [changed(Some(1))]);
    }

    #[sqlx::test]
    async fn failed_writes_send_nothing(pool: SqlitePool) {
        let state = state(&pool);
        let mut events = Box::pin(state.events.subscribe());
        let task_input =
            json!({ "name": "Vacuum", "cadence": { "amount": 7, "unit": "days" }, "priority": 0 });

        let (status, _) =
            request_with(&state, "PUT", "/api/tasks/99", Some(task_input.clone())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) =
            request_with(&state, "POST", "/api/projects/99/tasks", Some(task_input)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = request_with(&state, "DELETE", "/api/tasks/99", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = request_with(
            &state,
            "POST",
            "/api/executors",
            Some(json!({ "name": "" })),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(drain(&mut events), []);
    }

    #[sqlx::test]
    async fn lagging_clients_are_told_to_refetch_everything(pool: SqlitePool) {
        let state = state(&pool);
        let mut events = Box::pin(state.events.subscribe());
        for _ in 0..20 {
            state.events.changed(Some(1));
        }
        assert_eq!(events.next().await, Some(changed(None)));
    }

    #[sqlx::test]
    async fn endpoint_streams_events_until_shutdown(pool: SqlitePool) {
        let state = state(&pool);
        let response = response_with(&state, "GET", "/api/events", None).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/event-stream"
        );
        let mut body = response.into_body();
        let timeout = Duration::from_secs(5);

        state.events.changed(Some(1));
        let frame = tokio::time::timeout(timeout, body.frame()).await.unwrap();
        let data = frame.unwrap().unwrap().into_data().unwrap();
        assert_eq!(data, "data: {\"type\":\"changed\",\"project\":1}\n\n");

        state.events.shutdown();
        let frame = tokio::time::timeout(timeout, body.frame()).await.unwrap();
        assert!(frame.is_none());
    }
}
