use sea_orm_migration::prelude::*;
use crate::m20220907_223653_create_proxies::Proxy;
use crate::m20220913_213320_create_certificates::Certificate;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220914_000156_add_certificates_proxy"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Proxy::Table)
                    .add_column(ColumnDef::new(Alias::new("certificate"))
                        .string()
                        .not_null()
                    )
                    .add_foreign_key(TableForeignKey::new()
                        .name("fk-proxy-certificate-id")
                        .from_tbl(Proxy::Table)
                        .from_col(Alias::new("certificate"))
                        .to_tbl(Certificate::Table)
                        .to_col(Certificate::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned()
            )
            .await
    }
}