use sea_orm_migration::prelude::*;
use crate::m20220907_223633_create_teams::Team;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220907_223637_create_zones"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Zone::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Zone::Id)
                        .string()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(Zone::Owner)
                        .string()
                        .not_null()
                    )
                    .foreign_key(ForeignKey::create()
                        .name("fk-zone-owner-id")
                        .from(Zone::Table, Zone::Owner)
                        .to(Team::Table, Team::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(ColumnDef::new(Zone::Origin)
                        .string()
                        .not_null()
                        .unique_key()
                    )
                    .col(ColumnDef::new(Zone::Delegated)
                        .boolean()
                        .not_null()
                        .default(false)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Zone::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Zone {
    Table,
    Id,
    Owner,
    Origin,
    Delegated
}
