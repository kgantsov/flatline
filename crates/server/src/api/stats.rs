use axum::{extract::State, response::sse::{Event, Sse}};
use futures_util::stream::{self, Stream, StreamExt};
use std::{convert::Infallible, time::Duration};
use tokio_stream::wrappers::BroadcastStream;

use crate::AppState;
use shared::models::SseEvent;

fn sse_event_from(ev: &SseEvent) -> Option<Event> {
    let data = serde_json::to_string(ev).ok()?;
    // All events are sent as plain "message" events; the JSON "type" field
    // discriminates the variant so the client doesn't need named SSE events.
    Some(Event::default().data(data))
}

/// Stream real-time monitor events via Server-Sent Events.
/// Sends a StatsUpdate for every monitor immediately on connect, then streams
/// CheckResult, StatsUpdate, IncidentOpened, and IncidentResolved events live.
#[utoipa::path(
    get,
    path = "/api/v1/stats/stream",
    responses(
        (status = 200, description = "Server-sent event stream of monitor events", content_type = "text/event-stream"),
    ),
    tag = "stats"
)]
pub async fn stats_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Subscribe before snapshotting so we don't miss events that arrive between
    // the snapshot and the first broadcast message.
    let rx = state.event_tx.subscribe();

    let initial: Vec<Result<Event, Infallible>> = state
        .stats
        .iter()
        .filter_map(|entry| {
            let ev = SseEvent::StatsUpdate {
                monitor_id: *entry.key(),
                stats: entry.value().clone(),
            };
            sse_event_from(&ev).map(Ok)
        })
        .collect();

    let live = BroadcastStream::new(rx).filter_map(|msg| async move {
        msg.ok().and_then(|ev| sse_event_from(&ev).map(Ok))
    });

    let combined = stream::iter(initial).chain(live);

    Sse::new(combined).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
