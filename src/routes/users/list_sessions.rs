use axum::{Extension, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::entities::prelude::Session;
use crate::entities::session;
use crate::util::auth::UserFromBearer;
use sea_orm::{QueryFilter, ColumnTrait};
use serde::Serialize;

#[derive(Serialize)]
pub struct ListSessionsResponse {
    sessions: Vec<ListSession>
}

#[derive(Serialize)]
pub struct ListSession {
    id: String,
    name: String,
    ip: String
}

pub async fn list_sessions(
    Extension(ref connection): Extension<DatabaseConnection>,
    UserFromBearer(user): UserFromBearer,
) -> impl IntoResponse {
    let user = user.0;
    let mut listed_sessions: Vec<ListSession> = Vec::new();

    let total_sessions: Vec<session::Model> = Session::find()
        .filter(session::Column::Context.eq(user.clone().id))
        .all(connection)
        .await
        .expect("Failed to access database");

    for session in total_sessions {
        listed_sessions.push(ListSession {
            id: session.id,
            name: session.name,
            ip: session.ip
        });
    }

    (StatusCode::OK, Json(ListSessionsResponse { sessions: listed_sessions }))
}