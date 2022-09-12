use std::net::SocketAddr;
use axum::Extension;
use axum::extract::ConnectInfo;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::DatabaseConnection;

pub async fn rpc(
    Extension(ref connection): Extension<DatabaseConnection>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse{
    (StatusCode::INTERNAL_SERVER_ERROR, "NOT IMPLEMENTED")
}