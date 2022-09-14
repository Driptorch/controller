pub use sea_orm_migration::prelude::*;

mod m20220907_223615_create_users;
mod m20220907_223632_create_sessions;
mod m20220907_223633_create_teams;
mod m20220907_223634_create_team_members;
mod m20220907_223637_create_zones;
mod m20220907_223639_create_records;
mod m20220907_223653_create_proxies;
mod m20220908_204553_create_clients;
mod m20220913_213320_create_certificates;
mod m20220914_000156_add_certificates_proxy;
mod m20220914_000706_add_certificates_client;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220907_223615_create_users::Migration),
            Box::new(m20220907_223632_create_sessions::Migration),
            Box::new(m20220907_223633_create_teams::Migration),
            Box::new(m20220907_223634_create_team_members::Migration),
            Box::new(m20220907_223637_create_zones::Migration),
            Box::new(m20220907_223639_create_records::Migration),
            Box::new(m20220907_223653_create_proxies::Migration),
            Box::new(m20220908_204553_create_clients::Migration),
            Box::new(m20220913_213320_create_certificates::Migration),
            Box::new(m20220914_000156_add_certificates_proxy::Migration),
            Box::new(m20220914_000706_add_certificates_client::Migration),
        ]
    }
}
