//! HTTP handlers for events API

use crate::dapr::{DaprSubscription, DaprSubscriptionResponse, DaprTopicEvent};
use crate::error::EventError;
use crate::models::{CreateEvent, Event, EventFilter, EventStats};
use crate::repository::EventRepository;
use crate::service::{EventService, HealthStatus};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

/// Events router state
pub type EventsState<R> = Arc<EventService<R>>;

/// Create the events router
pub fn events_router<R: EventRepository + 'static>() -> Router<EventsState<R>> {
    Router::new()
        // CRUD endpoints
        .route("/", get(list_events::<R>).post(create_event::<R>))
        .route("/batch", post(create_events_batch::<R>))
        .route("/count", get(count_events::<R>))
        .route("/stats", get(get_stats::<R>))
        .route("/{id}", get(get_event::<R>).delete(delete_event::<R>))
        // Dapr subscription endpoints
        .route("/subscribe", post(handle_event_subscription::<R>))
        .route("/dapr/subscribe", get(get_subscriptions))
        // Health
        .route("/health", get(health_check::<R>))
}

/// List events with filtering
#[utoipa::path(
    get,
    path = "/",
    params(
        ("category" = Option<String>, Query, description = "Filter by category"),
        ("severity" = Option<String>, Query, description = "Filter by severity"),
        ("name" = Option<String>, Query, description = "Filter by event name"),
        ("source" = Option<String>, Query, description = "Filter by source"),
        ("from" = Option<String>, Query, description = "Start time (ISO 8601)"),
        ("to" = Option<String>, Query, description = "End time (ISO 8601)"),
        ("search" = Option<String>, Query, description = "Search in message"),
        ("offset" = Option<u64>, Query, description = "Pagination offset"),
        ("limit" = Option<u64>, Query, description = "Pagination limit"),
    ),
    responses(
        (status = 200, description = "List of events", body = Vec<Event>),
        (status = 500, description = "Internal error")
    ),
    tag = "events"
)]
#[instrument(skip(state))]
pub async fn list_events<R: EventRepository>(
    State(state): State<EventsState<R>>,
    Query(filter): Query<EventFilter>,
) -> Result<Json<Vec<Event>>, EventError> {
    let events = state.list(&filter).await?;
    Ok(Json(events))
}

/// Create a new event
#[utoipa::path(
    post,
    path = "/",
    request_body = CreateEvent,
    responses(
        (status = 201, description = "Event created", body = Event),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal error")
    ),
    tag = "events"
)]
#[instrument(skip(state, create), fields(event_name = %create.name))]
pub async fn create_event<R: EventRepository>(
    State(state): State<EventsState<R>>,
    Json(create): Json<CreateEvent>,
) -> Result<impl IntoResponse, EventError> {
    let event = state.create(create).await?;
    Ok((StatusCode::CREATED, Json(event)))
}

/// Create multiple events in batch
#[utoipa::path(
    post,
    path = "/batch",
    request_body = Vec<CreateEvent>,
    responses(
        (status = 201, description = "Events created", body = Vec<Event>),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal error")
    ),
    tag = "events"
)]
#[instrument(skip(state, creates), fields(count = creates.len()))]
pub async fn create_events_batch<R: EventRepository>(
    State(state): State<EventsState<R>>,
    Json(creates): Json<Vec<CreateEvent>>,
) -> Result<impl IntoResponse, EventError> {
    let events = state.create_batch(creates).await?;
    Ok((StatusCode::CREATED, Json(events)))
}

/// Get event by ID
#[utoipa::path(
    get,
    path = "/{id}",
    params(
        ("id" = Uuid, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Event found", body = Event),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Internal error")
    ),
    tag = "events"
)]
#[instrument(skip(state))]
pub async fn get_event<R: EventRepository>(
    State(state): State<EventsState<R>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Event>, EventError> {
    let event = state.get_by_id(&id).await?;
    Ok(Json(event))
}

/// Delete event by ID
#[utoipa::path(
    delete,
    path = "/{id}",
    params(
        ("id" = Uuid, Path, description = "Event ID")
    ),
    responses(
        (status = 204, description = "Event deleted"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Internal error")
    ),
    tag = "events"
)]
#[instrument(skip(state))]
pub async fn delete_event<R: EventRepository>(
    State(state): State<EventsState<R>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, EventError> {
    let deleted = state.delete(&id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(EventError::NotFound { id: id.to_string() })
    }
}

/// Count events matching filter
#[utoipa::path(
    get,
    path = "/count",
    params(
        ("category" = Option<String>, Query, description = "Filter by category"),
        ("severity" = Option<String>, Query, description = "Filter by severity"),
    ),
    responses(
        (status = 200, description = "Event count", body = CountResponse),
        (status = 500, description = "Internal error")
    ),
    tag = "events"
)]
#[instrument(skip(state))]
pub async fn count_events<R: EventRepository>(
    State(state): State<EventsState<R>>,
    Query(filter): Query<EventFilter>,
) -> Result<Json<CountResponse>, EventError> {
    let count = state.count(&filter).await?;
    Ok(Json(CountResponse { count }))
}

/// Get event statistics
#[utoipa::path(
    get,
    path = "/stats",
    responses(
        (status = 200, description = "Event statistics", body = EventStats),
        (status = 500, description = "Internal error")
    ),
    tag = "events"
)]
#[instrument(skip(state))]
pub async fn get_stats<R: EventRepository>(
    State(state): State<EventsState<R>>,
) -> Result<Json<EventStats>, EventError> {
    let stats = state.stats().await?;
    Ok(Json(stats))
}

/// Handle incoming events from Dapr pub/sub subscription
#[instrument(skip(state, topic_event), fields(event_id = %topic_event.id))]
async fn handle_event_subscription<R: EventRepository>(
    State(state): State<EventsState<R>>,
    Json(topic_event): Json<DaprTopicEvent<CreateEvent>>,
) -> Json<DaprSubscriptionResponse> {
    info!(
        topic = %topic_event.topic,
        source = %topic_event.source,
        "Received event from Dapr subscription"
    );

    // Process the event
    match state.create(topic_event.data).await {
        Ok(event) => {
            info!(event_id = %event.id, "Event processed successfully");
            Json(DaprSubscriptionResponse::success())
        }
        Err(e) => {
            warn!(error = %e, "Failed to process event");
            // Retry on transient errors
            Json(DaprSubscriptionResponse::retry())
        }
    }
}

/// Get Dapr subscriptions (called by Dapr sidecar on startup)
async fn get_subscriptions() -> Json<Vec<DaprSubscription>> {
    Json(vec![DaprSubscription {
        pubsubname: "events-pubsub".to_string(),
        topic: "events".to_string(),
        route: "/api/events/subscribe".to_string(),
    }])
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health status", body = HealthStatus),
        (status = 500, description = "Unhealthy")
    ),
    tag = "events"
)]
pub async fn health_check<R: EventRepository>(
    State(state): State<EventsState<R>>,
) -> Result<Json<HealthStatus>, EventError> {
    let status = state.health().await?;
    Ok(Json(status))
}

/// Count response
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CountResponse {
    pub count: u64,
}
