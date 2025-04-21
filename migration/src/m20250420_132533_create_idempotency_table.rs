use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
enum HeaderPairIden {
    #[sea_orm(iden = "header_pair")]
    HeaderPair,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderPairRecord {
    pub name: String,
    pub value: Vec<u8>,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(r#"CREATE TYPE header_pair AS (name TEXT, value BYTEA)"#)
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Idempotency::Table)
                    .if_not_exists()
                    .col(uuid(Idempotency::UserId).not_null())
                    .col(string(Idempotency::IdempotencyKey).not_null())
                    .col(small_integer(Idempotency::ResponseStatusCode).not_null())
                    .col(
                        array(
                            Idempotency::ResponseHeaders,
                            ColumnType::Custom(SeaRc::new(HeaderPairIden::HeaderPair)),
                        )
                        .not_null(),
                    )
                    .col(binary(Idempotency::ResponseBody).not_null())
                    .col(timestamp_with_time_zone(Idempotency::CreatedAt).not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_idempotency")
                            .col(Idempotency::UserId)
                            .col(Idempotency::IdempotencyKey),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Idempotency::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS header_pair")
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Idempotency {
    Table,
    UserId,
    IdempotencyKey,
    ResponseStatusCode,
    ResponseHeaders,
    ResponseBody,
    CreatedAt,
}
