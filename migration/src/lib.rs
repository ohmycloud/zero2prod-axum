pub use sea_orm_migration::prelude::*;

mod m20250116_212701_create_subscriptions_table;
mod m20250218_152412_create_subscription_tokens_table;
mod m20250223_072332_create_users_table;
mod m20250419_075152_add_seed_user;
mod m20250420_132533_create_idempotency_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250116_212701_create_subscriptions_table::Migration),
            Box::new(m20250218_152412_create_subscription_tokens_table::Migration),
            Box::new(m20250223_072332_create_users_table::Migration),
            Box::new(m20250419_075152_add_seed_user::Migration),
            Box::new(m20250420_132533_create_idempotency_table::Migration),
        ]
    }
}
