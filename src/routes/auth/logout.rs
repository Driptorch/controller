use axum::{Extension, extract};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::util::auth::UserFromBearer;
use serde::Deserialize;
use crate::entities::prelude::Session;
use crate::entities::session;

#[derive(Deserialize)]
pub struct LogoutInput {
    session_id: String
}

pub async fn logout(
    Extension(ref connection): Extension<DatabaseConnection>,
    UserFromBearer(user): UserFromBearer,
    extract::Json(payload): extract::Json<LogoutInput>
) -> impl IntoResponse {
    let accessed_session_id = user.1;
    let user = user.0;

    return match payload.session_id.as_str() {
        "ALL" => {
            // Invalidate all of the user's sessions
            // By connecting to this route, we know there is at least 1 session
            session::Entity::delete_many()
                .filter(session::Column::Context.eq(user.clone().id))
                .exec(connection)
                .await
                .expect("Failed to run delete on sessions database");

            // Ensure no sessions are left for the user
            let total_sessions = Session::find()
                .filter(session::Column::Context.eq(user.clone().id))
                .all(connection)
                .await
                .expect("Failed to access database");

            if total_sessions.is_empty() {
                (StatusCode::OK, format!("Goodbye {}!", user.name))
            } else {
                error!("Unable to delete {}'s sessions!", user.id);
                (StatusCode::INTERNAL_SERVER_ERROR, "An Internal Server Error has occurred".to_string())
            }
        }
        "" => {
            // Invalidate the user's current session
            let session_deletion = session::Entity::delete_by_id(accessed_session_id.clone())
                .exec(connection)
                .await
                .expect("Failed to run delete on sessions database");

            if session_deletion.rows_affected.eq(&1) {
                (StatusCode::OK, format!("Goodbye {}!", user.name))
            } else {
                error!("Unable to delete {}'s current session! {}", user.id, accessed_session_id);
                (StatusCode::INTERNAL_SERVER_ERROR, "An Internal Server Error has occurred".to_string())
            }
        }
        _ => {
            // Check if the ID is valid and exists
            let requested_session = Session::find()
                .filter(session::Column::Id.eq(payload.session_id.clone()))
                .one(connection)
                .await;

            match requested_session {
                Ok(_) => {
                    let requested_session = requested_session.unwrap();
                    if requested_session.is_some() {
                        let requested_session = requested_session.unwrap();
                        // Ensure the session is for the currently authed user
                        if requested_session.context != user.id {
                            return (StatusCode::BAD_REQUEST, "Requested session ID is invalid".to_string());
                        }

                        // Invalidate the session
                        let session_deletion = session::Entity::delete_by_id(requested_session.clone().id)
                            .exec(connection)
                            .await
                            .expect("Failed to run delete on sessions database");

                        if session_deletion.rows_affected.eq(&1) {
                            (StatusCode::OK, format!("Goodbye {}!", user.name))
                        } else {
                            error!("Unable to delete {}'s requested session! {}", user.id, requested_session.clone().id);
                            (StatusCode::INTERNAL_SERVER_ERROR, "An Internal Server Error has occurred".to_string())
                        }
                    } else {
                        (StatusCode::BAD_REQUEST, "Requested session ID is invalid".to_string())
                    }
                }
                Err(_) => {
                    (StatusCode::BAD_REQUEST, "Requested session ID is invalid".to_string())
                }
            }
        }
    }
}