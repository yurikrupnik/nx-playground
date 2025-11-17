use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Todos::Table)
                    .if_not_exists()
                    .col(pk_uuid(Todos::Id))
                    .col(string(Todos::Name))
                    .col(string_null(Todos::Description))
                    .col(boolean(Todos::Completed).default(false))
                    .col(timestamp_with_time_zone(Todos::CreatedAt))
                    .col(timestamp_with_time_zone(Todos::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Todos::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Todos {
    Table,
    Id,
    Name,
    Description,
    Completed,
    CreatedAt,
    UpdatedAt,
}
