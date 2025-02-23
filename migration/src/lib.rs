pub use sea_orm_migration::prelude::*;

mod m20250116_212701_create_table;
mod m20250218_152412_create_subscription_tokens;
mod m20250223_072332_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250116_212701_create_table::Migration),
            Box::new(m20250218_152412_create_subscription_tokens::Migration),
            Box::new(m20250223_072332_create_users_table::Migration),
        ]
    }
}
