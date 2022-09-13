use std::net::SocketAddr;
use axum::{Extension, Form, Json};
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use chrono::{Duration, NaiveDateTime};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use lazy_static::lazy_static;
use regex::Regex;

use crate::entities::{team, team_member, user};
use crate::entities::prelude::{Team, TeamMember};
use crate::entities::user::Entity as User;

use crate::entities::session;
use crate::entities::session::Entity as Session;

use crate::util::{generate_session_token, hash_password};
use crate::util::auth::{assemble_session_name, TeamPermissions};

#[derive(Deserialize, Clone)]
pub struct NewUserForm {
    name: String,
    email: String,
    password: String
}

#[derive(Serialize)]
pub struct NewUserFormIssues {
    name: Vec<String>,
    email: Vec<String>,
    password: Vec<String>
}

#[derive(Serialize)]
pub struct NewUserFormResponse {
    session_token: Option<String>,
    issues: Option<NewUserFormIssues>
}

pub async fn register(
    Extension(ref connection): Extension<DatabaseConnection>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(input): Form<NewUserForm>
) -> impl IntoResponse {
    let mut validation_issues = NewUserFormIssues {
        name: vec![],
        email: vec![],
        password: vec![]
    };

    lazy_static! {
        static ref EMAIL_RE: Regex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#)
        .expect("Failed to compile email regex");
    }

    // Ensure form contents are safe
    if input.name.is_empty() {
        validation_issues.name.push("Name cannot be empty.".to_string());
    } else if input.name.len() > 128 {
        validation_issues.name.push("Name is too long.".to_string());
    }

    if input.email.is_empty() {
        validation_issues.email.push("Email cannot be empty.".to_string());
    } else if !EMAIL_RE.is_match(&input.email) {
        validation_issues.email.push("Email is invalid.".to_string());
    }

    if input.password.is_empty(){
        validation_issues.password.push("Password cannot be empty.".to_string());
    } else {
        // Check password security
        let pass_estimate = zxcvbn::zxcvbn(&input.password, &[])
            .expect("Failed to check password security.");

        if pass_estimate.score() <= 2 {
            for i in pass_estimate.feedback().clone().unwrap().suggestions() {
                validation_issues.password.push(i.to_string());
            }
        }
    }

    // Return early if we have issues with form content so far
    if !validation_issues.name.is_empty() || !validation_issues.email.is_empty() || !validation_issues.password.is_empty()  {
        return (StatusCode::BAD_REQUEST, Json(NewUserFormResponse { session_token: None, issues: Some(validation_issues) }));
    }

    // Check to see if a user with the email already exists
    let existing_user = User::find()
        .filter(user::Column::Email.eq(input.email.clone()))
        .one(connection)
        .await
        .expect("Failed to check database.");

    if existing_user.is_some() {
        validation_issues.email.push("An account with this email already exists.".to_string());

        return (StatusCode::BAD_REQUEST, Json(NewUserFormResponse { session_token: None, issues: Some(validation_issues) }));
    }

    let user_id = Ulid::new().to_string();

    let new_user = user::ActiveModel {
        id: ActiveValue::Set(user_id.clone()),
        name: ActiveValue::Set(input.clone().name),
        email: ActiveValue::Set(input.clone().email),
        password: ActiveValue::Set(hash_password(input.clone().password)),
        ..Default::default()
    };

    let res = User::insert(new_user.clone())
        .exec(connection)
        .await;

    return match res {
        Ok(_) => {
            // Create a personal team
            let team_id = String::from(Ulid::new());

            let new_team = team::ActiveModel {
                id: ActiveValue::set(team_id.clone()),
                name: ActiveValue::Set(
                    format!("{}'s Personal Team", &input.name)
                ),
                active: Default::default(),
                personal: ActiveValue::Set(true)
            };

            let team_creation = Team::insert(new_team.clone())
                .exec(connection)
                .await;

            if team_creation.is_err() {
                // Delete user on team creation error
                new_user.delete(connection).await
                    .expect("Failed to delete user from database after failing to create personal team for said user!");

                return (StatusCode::INTERNAL_SERVER_ERROR, Json(NewUserFormResponse { session_token: None, issues: None }));
            }

            // Add user to personal team
            let team_addition = TeamMember::insert(
                team_member::ActiveModel {
                    id: ActiveValue::Set(String::from(Ulid::new())),
                    team_id: ActiveValue::Set(team_id.clone()),
                    user_id: ActiveValue::Set(user_id.clone()),
                    permission: ActiveValue::Set(TeamPermissions::OWNER.to_string())
                }
            ).exec(connection)
                .await;

            if team_addition.is_err() {
                // Delete team and user on permission addition error
                new_team.delete(connection).await
                    .expect("Failed to delete team from database after failing to add permissions for personal team for said user!");
                new_user.delete(connection).await
                    .expect("Failed to delete user from database after failing to add permissions for personal team for said user!");

                return (StatusCode::INTERNAL_SERVER_ERROR, Json(NewUserFormResponse { session_token: None, issues: None }));
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
                context: ActiveValue::Set(user_id.clone()),
                expiry: ActiveValue::Set(expiry)
            };

            let session_res = Session::insert(new_session)
                .exec(connection)
                .await;

            match session_res {
                Ok(_) => {
                    (StatusCode::CREATED, Json(NewUserFormResponse { session_token: Some(session_token), issues: None }))
                }
                Err(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(NewUserFormResponse { session_token: None, issues: None }))
                }
            }
        }
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(NewUserFormResponse { session_token: None, issues: None }))
        }
    }
}