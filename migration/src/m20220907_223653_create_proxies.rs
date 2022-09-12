use sea_orm_migration::prelude::*;
use crate::m20220907_223639_create_records::Record;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220907_223653_create_proxies"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Proxy::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Proxy::Id)
                        .string()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(Proxy::Record)
                        .string()
                        .not_null()
                    )
                    .foreign_key(ForeignKey::create()
                        .name("fk-proxy-record-id")
                        .from(Proxy::Table, Proxy::Record)
                        .to(Record::Table, Record::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(ColumnDef::new(Proxy::Port)
                        .integer()
                        .not_null()
                        .default(443)
                    )
                    .col(ColumnDef::new(Proxy::Active)
                        .boolean()
                        .not_null()
                        .default(true)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Proxy::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Proxy {
    Table,
    Id,
    Record,
    Port,
    Active,
}
