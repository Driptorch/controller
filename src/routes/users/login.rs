use std::net::SocketAddr;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Extension, Form, Json};
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use chrono::{Duration, NaiveDateTime};
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::entities::session;
use crate::entities::session::Entity as Session;
use crate::entities::user;
use crate::entities::user::Entity as User;
use crate::util::auth::assemble_session_name;
use crate::util::generate_session_token;

#[derive(Deserialize)]
pub struct AuthUserForm {
    email: String,
    password: String
}

#[derive(Serialize)]
pub struct AuthUserFormIssues {
    email: Vec<String>,
    password: Vec<String>
}

#[derive(Serialize)]
pub struct AuthUserFormResponse {
    session_token: Option<String>,
    issues: Option<AuthUserFormIssues>
}

pub async fn login(
    Extension(ref connection): Extension<DatabaseConnection>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(input): Form<AuthUserForm>
) -> impl IntoResponse {
    let mut validation_issues = AuthUserFormIssues {
        email: vec![],
        password: vec![]
    };

    lazy_static! {
        static ref EMAIL_RE: Regex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#)
        .expect("Failed to compile email regex");
    }

    // Ensure form contents are safe
    if input.email.is_empty() {
        validation_issues.email.push("Email cannot be empty.".to_string());
    } else if !EMAIL_RE.is_match(&input.email) {
        validation_issues.email.push("Email is invalid.".to_string());
    }

    if input.password.is_empty(){
        validation_issues.password.push("Password cannot be empty.".to_string());
    }

    // Return early if we have issues with form content so far
    if !validation_issues.email.is_empty() || !validation_issues.password.is_empty()  {
        return (StatusCode::BAD_REQUEST, Json(AuthUserFormResponse { session_token: None, issues: Some(validation_issues) }));
    }

    // Check to see if a user with the email exists
    let existing_user: Option<user::Model> = User::find()
        .filter(user::Column::Email.eq(input.email.clone()))
        .one(connection)
        .await
        .expect("Failed to check database.");

    if !existing_user.is_some() {
        validation_issues.email.push("An account with this email doesn't exist.".to_string());

        return (StatusCode::BAD_REQUEST, Json(AuthUserFormResponse { session_token: None, issues: Some(validation_issues) }));
    }

    let existing_user = existing_user.unwrap();

    // Check password
    let existing_password_hash = PasswordHash::new(&existing_user.password)
        .expect("Failed to generate password hash from database.");
    if !Argon2::default().verify_password(input.password.as_bytes(), &existing_password_hash).is_ok() {
        validation_issues.password.push("Incorrect password.".to_string());
        return (StatusCode::BAD_REQUEST, Json(AuthUserFormResponse { session_token: None, issues: Some(validation_issues) }));
    }

    let session_name;
    match headers.get("User-Agent") {
        None => {
            session_name = String::from("Unknown");
        }
        Some(header) => {
            match header.to_str() {
                Ok(header) => {
                    session_name = assemble_session_name(header).await;
                }
                Err(_) => {
                    session_name = String::from("Unknown");
                }
            }
        }
    }

    let ip;
    match headers.get("X-Real-IP") {
        None => {
            ip = addr.ip().to_string();
        }
        Some(header) => {
            match header.to_str() {
                Ok(header) => {
                    ip = String::from(header);
                }
                Err(_) => {
                    ip = addr.ip().to_string();
                }
            }
        }
    }

    let session_token = generate_session_token();
    let expiry: NaiveDateTime = chrono::offset::Utc::now().naive_local() + Duration::days(20);

    // Generate session for newly created user
    let new_session = session::ActiveModel {
        id: ActiveValue::Set(Ulid::new().to_string()),
        name: ActiveValue::Set(session_name.clone()),
        ip: ActiveValue::set(ip.clone()),
        token: ActiveValue::Set(session_token.clone()),
        context: ActiveValue::Set(existing_user.id.clone()),
        expiry: ActiveValue::Set(expiry)
    };

    let session_res = Session::insert(new_session)
        .exec(connection)
        .await;

    match session_res {
        Ok(_) => {
            (StatusCode::OK, Json(AuthUserFormResponse { session_token: Some(session_token), issues: None }))
        }
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthUserFormResponse { session_token: None, issues: None }))
        }
    }
}