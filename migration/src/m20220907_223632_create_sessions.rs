use sea_orm_migration::prelude::*;

use super::m20220907_223615_create_users::User;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220907_223632_create_sessions"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Session::Id)
                        .string()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(Session::Name)
                        .string()
                        .not_null()
                    )
                    .col(ColumnDef::new(Session::Ip)
                        .string()
                        .not_null()
                    )
                    .col(ColumnDef::new(Session::Token)
                        .string()
                        .not_null()
                    )
                    .col(ColumnDef::new(Session::Context)
                        .string()
                        .not_null()
                    )
                    .foreign_key(ForeignKey::create()
                        .name("fk-sessions-user-id")
                        .from(Session::Table, Session::Context)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(ColumnDef::new(Session::Expiry)
                        .timestamp()
                        .not_null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Session {
    Table,
    Id,
    Name,
    Ip,
    Token,
    Context,
    Expiry,
}
