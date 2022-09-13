use std::borrow::Cow;
use std::env;
use std::fmt;
use std::fmt::Formatter;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{header::AUTHORIZATION, StatusCode};
use axum::http::request::Parts;
use lazy_static::lazy_static;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use user_agent_parser::{OS, Product};
use user_agent_parser::UserAgentParser;

use crate::entities::{session, user};
use crate::entities::prelude::{Session, User};

pub enum TeamPermissions {
    OWNER,
    ADMIN,
    EDITOR,
    VIEWER
}

impl fmt::Display for TeamPermissions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TeamPermissions::OWNER => write!(f, "OWNER"),
            TeamPermissions::ADMIN => write!(f, "ADMIN"),
            TeamPermissions::EDITOR => write!(f, "EDITOR"),
            TeamPermissions::VIEWER => write!(f, "VIEWER")
        }
    }
}

/// Gets a user model and session id from a supplied session token
pub async fn get_user_from_token(token: String, connection: &DatabaseConnection) -> Option<(user::Model, String)> {
    let requested_session: Option<session::Model> = Session::find()
        .filter(session::Column::Token.eq(token))
        .one(connection)
        .await
        .expect("Failed to retrieve session from the database.");

    return match requested_session {
        None => {
            None
        }
        Some(_) => {
            let requested_session = requested_session.unwrap();

            let contexted_user: Option<user::Model> = User::find()
                .filter(user::Column::Id.eq(requested_session.clone().context))
                .one(connection)
                .await
                .expect("Failed to retrieve user from the database.");

            match contexted_user {
                None => {
                    error!("Session {} still exists for user {} of which doesn't exist!", requested_session.id, requested_session.context);
                    None
                }
                Some(_) => {
                    Some((contexted_user.unwrap(), requested_session.id))
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct UserFromBearer(pub (user::Model, String));

#[async_trait]
impl<S> FromRequestParts<S> for UserFromBearer
    where
        S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get authorisation header
        let authorisation = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or((StatusCode::UNAUTHORIZED, "`Authorization` header is missing"))?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "`Authorization` header contains invalid characters",
                )
            })?;

        // Check that its a well-formed bearer and return
        let split = authorisation.split_once(' ');
        match split {
            Some((name, contents)) if name == "Bearer" => {
                // Get database connection from header
                let connection: &DatabaseConnection = parts.extensions.get::<DatabaseConnection>()
                    .expect("Failed to get database connection from users extractor");

                match get_user_from_token(contents.to_string(), connection).await {
                    None => {
                        Err((StatusCode::UNAUTHORIZED, "Provided token is invalid"))
                    }
                    Some(user) => Ok(Self(user))
                }
            },
            _ => Err((
                StatusCode::BAD_REQUEST,
                "`Authorization` header must be a bearer token",
            )),
        }
    }
}

pub async fn assemble_session_name(header: &str) -> String {
    // TODO: Maybe reconsider having a static session name?

    lazy_static! {
        static ref UAP: UserAgentParser = UserAgentParser::from_path(&env::var("UAP_REGEXES").unwrap_or(String::from("./regexes.yaml"))).expect("Failed to load regexes.yaml");
    }

    let product: Product = UAP.parse_product(&header);
    let os: OS = UAP.parse_os(&header);

    let mut session_name_builder: String = String::new();

    session_name_builder.push_str(
        &product.name.unwrap_or(Cow::from("Unknown"))
    );

    if product.major.is_some() {
        session_name_builder.push_str(" ");

        session_name_builder.push_str(
            &product.major.unwrap()
        )
    }

    session_name_builder.push_str(" / ");

    session_name_builder.push_str(
        &os.name.unwrap_or(Cow::from("Unknown"))
    );

    if os.major.is_some() {
        session_name_builder.push_str(" ");

        session_name_builder.push_str(
            &os.major.unwrap()
        )
    };

    return session_name_builder
}