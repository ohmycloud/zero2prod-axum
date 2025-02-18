use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250116_212701_create_table::Subscriptions;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SubscriptionTokens::Table)
                    .if_not_exists()
                    .col(
                        string(SubscriptionTokens::SubscriptionToken)
                            .not_null()
                            .primary_key(),
                    )
                    .col(uuid(SubscriptionTokens::SubscriptionId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscription_id")
                            .from(
                                SubscriptionTokens::Table,
                                SubscriptionTokens::SubscriptionId,
                            )
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SubscriptionTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SubscriptionTokens {
    Table,
    SubscriptionToken,
    SubscriptionId,
}
