use sea_orm_migration::prelude::*;
use crate::m20220907_223615_create_users::User;
use crate::m20220907_223633_create_teams::Team;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220907_223634_create_team_members"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TeamMember::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TeamMember::Id)
                        .string()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(TeamMember::TeamId)
                        .string()
                        .not_null()
                    )
                    .foreign_key(ForeignKey::create()
                        .name("fk-team-members-team-id")
                        .from(TeamMember::Table, TeamMember::TeamId)
                        .to(Team::Table, Team::Id)
                    )
                    .col(ColumnDef::new(TeamMember::UserId)
                        .string()
                        .not_null()
                    )
                    .foreign_key(ForeignKey::create()
                        .name("fk-team-members-user-id")
                        .from(TeamMember::Table, TeamMember::UserId)
                        .to(User::Table, User::Id)
                    )
                    .col(ColumnDef::new(TeamMember::Permission)
                        .string()
                        .not_null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TeamMember::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum TeamMember {
    Table,
    Id,
    TeamId,
    UserId,
    Permission
}
