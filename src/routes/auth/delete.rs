use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::entities::prelude::{Team, TeamMember};
use crate::entities::{team, team_member, user};
use crate::util::auth::{TeamPermissions, UserFromBearer};

pub async fn delete(
    Extension(ref connection): Extension<DatabaseConnection>,
    UserFromBearer(user): UserFromBearer,
) -> impl IntoResponse {
    let user = user.0;

    let owned_teams = TeamMember::find()
        .filter(team_member::Column::UserId.eq(user.clone().id))
        .filter(team_member::Column::Permission.eq(TeamPermissions::OWNER.to_string()))
        .all(connection)
        .await
        .expect("Failed to retrieve teams from database");

    let mut personal_team: Option<team::Model> = None;

    for team in owned_teams {
        let owned_team = Team::find_by_id(team.clone().team_id)
            .one(connection)
            .await
            .expect("Failed to get owned team by id");

        match owned_team {
            None => {
                error!("team_member still exists for {} for {} but the team doesn't exist!", team.team_id, user.id);

                return (StatusCode::INTERNAL_SERVER_ERROR, "An internal server error has occurred".to_string());
            }
            Some(owned_team) => {
                if !owned_team.personal {
                    return (StatusCode::BAD_REQUEST, "User owns non-personal teams.".to_string());
                } else {
                    personal_team = Some(owned_team.clone());
                }
            }
        }
    }

    let personal_team = personal_team.unwrap();

    // Kick everyone out of the user's personal team
    let team_member_delete = team_member::Entity::delete_many()
        .filter(team_member::Column::TeamId.eq(personal_team.clone().id))
        .exec(connection)
        .await
        .expect("Could not delete team_member during user deletion!");

    if team_member_delete.rows_affected.eq(&0) {
        error!("Could not delete personal team_member for {}!", user.id);
        return (StatusCode::INTERNAL_SERVER_ERROR, "An internal server error has occurred".to_string());
    }

    let team_delete = team::Entity::delete_by_id(personal_team.clone().id)
        .exec(connection)
        .await
        .expect("Failed to delete personal team during user deletion!");

    if team_delete.rows_affected.eq(&0) {
        error!("Could not delete personal team for {}!", user.id);
        return (StatusCode::INTERNAL_SERVER_ERROR, "An internal server error has occurred".to_string());
    }

    let user_delete = user::Entity::delete_by_id(user.clone().id)
        .exec(connection)
        .await
        .expect("Failed to delete user!");

    if user_delete.rows_affected.eq(&0) {
        error!("Could not delete user {}!", user.id);
        return (StatusCode::INTERNAL_SERVER_ERROR, "An internal server error has occurred".to_string());
    }

    (StatusCode::OK, format!("Goodbye forever, {}!", user.name))
}