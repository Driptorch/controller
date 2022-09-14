use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220913_213320_create_certificates"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Certificate::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Certificate::Id)
                        .string()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(Certificate::Data)
                        .binary()
                        .not_null()
                    )
                    .col(ColumnDef::new(Certificate::Key)
                        .binary()
                        .not_null()
                    )
                    .col(ColumnDef::new(Certificate::Nonce)
                        .binary()
                        .not_null()
                        .unique_key()
                    )
                    .col(ColumnDef::new(Certificate::CertType)
                        .string()
                        .not_null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Certificate::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Certificate {
    Table,
    Id,
    Data,
    Key,
    Nonce,
    CertType
}
