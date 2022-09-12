use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220907_223633_create_teams"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Team::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Team::Id)
                        .string()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(Team::Name)
                        .string()
                        .not_null()
                    )
                    .col(ColumnDef::new(Team::Active)
                        .boolean()
                        .not_null()
                        .default(true)
                    )
                    .col(ColumnDef::new(Team::Personal)
                        .boolean()
                        .not_null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Team::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Team {
    Table,
    Id,
    Name,
    Active,
    Personal
}
