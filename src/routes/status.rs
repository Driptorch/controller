use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::{Serialize, Deserialize};
use crate::VERSION;

#[derive(Serialize, Deserialize)]
enum Health {
    HEALTHY,
    UNHEALTHY,
    DEAD
}

#[derive(Serialize, Deserialize)]
enum Context {
    CONTROLLER,
    CLIENT
}

#[derive(Serialize)]
struct Status {
    pub context: Context,
    pub version: String,
    pub health: Health
}

pub async fn status() -> impl IntoResponse {
    (StatusCode::OK, Json(Status {
        context: Context::CONTROLLER,
        version: VERSION.to_string(),
        health: Health::HEALTHY
    })
    )
}