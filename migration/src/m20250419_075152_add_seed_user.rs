use sea_orm_migration::prelude::*;
use uuid::Uuid;

use crate::m20250223_072332_create_users_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let user_id = Uuid::parse_str("ddf8994f-d522-4659-8d02-c1d479057be6").unwrap();
        // 插入数据
        let insert = Query::insert()
            .into_table(Users::Table)
            .columns([Users::UserId, Users::Username, Users::PasswordHash])
            .values_panic([
                user_id.into(),
                "admin".into(),
                "$argon2id$v=19$m=15000,t=2,p=1$OEx/rcq+3ts//WUDzGNl2g$Am8UFBA4w5NJEmAtquGvBmAlu92q/VQcaoL5AyJPfc8".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;
        Ok(())
    }
}
