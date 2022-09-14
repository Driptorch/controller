use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220908_204553_create_clients"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Client::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Client::Id)
                        .string()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(Client::Name)
                        .string()
                        .not_null()
                        .unique_key()
                    )
                    .col(ColumnDef::new(Client::Ip)
                        .string()
                        .not_null()
                    )
                    .col(ColumnDef::new(Client::Key)
                        .string()
                        .not_null()
                    )
                    .col(ColumnDef::new(Client::Active)
                        .boolean()
                        .not_null()
                        .default(true)
                    )
                    .col(ColumnDef::new(Client::DNS)
                        .boolean()
                        .not_null()
                        .default(true)
                    )
                    .col(ColumnDef::new(Client::Proxy)
                        .boolean()
                        .not_null()
                        .default(true)
                    )
                    .col(ColumnDef::new(Client::Health)
                        .string()
                        .not_null()
                        .default("DEAD")
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Client::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Client {
    Table,
    Id,
    Name,
    Ip,
    Key,
    Active,
    DNS,
    Proxy,
    Health
}
