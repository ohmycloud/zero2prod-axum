use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .if_not_exists()
                    .col(pk_uuid(Subscriptions::Id))
                    .col(text_uniq(Subscriptions::Email))
                    .col(text(Subscriptions::Name))
                    .col(timestamp_with_time_zone(Subscriptions::SubscribedAt))
                    .col(text(Subscriptions::Status))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Subscriptions {
    Table,
    Id,
    Email,
    Name,
    SubscribedAt,
    Status,
}
