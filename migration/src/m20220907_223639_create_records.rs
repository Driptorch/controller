use sea_orm_migration::prelude::*;
use crate::m20220907_223637_create_zones::Zone;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220907_223639_create_records"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Record::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Record::Id)
                        .string()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(Record::Zone)
                        .string()
                        .not_null()
                    )
                    .foreign_key(ForeignKey::create()
                        .name("fk-record-zone-id")
                        .from(Record::Table, Record::Zone)
                        .to(Zone::Table, Zone::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(ColumnDef::new(Record::Value)
                        .binary()
                        .not_null()
                    )
                    .col(ColumnDef::new(Record::Active)
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
            .drop_table(Table::drop().table(Record::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Record {
    Table,
    Id,
    Zone,
    Value,
    Active,
}
