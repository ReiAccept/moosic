pub use sea_orm_migration::prelude::*;

mod m20260614_000001_create_users_table;
mod m20260617_000002_create_remaining_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260614_000001_create_users_table::Migration),
            Box::new(m20260617_000002_create_remaining_tables::Migration),
        ]
    }
}
