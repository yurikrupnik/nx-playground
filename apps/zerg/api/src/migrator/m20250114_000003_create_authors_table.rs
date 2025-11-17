use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Authors::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Authors::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Authors::Name).string().not_null())
                    .col(ColumnDef::new(Authors::Bio).text())
                    .col(
                        ColumnDef::new(Authors::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Authors::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on name for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_authors_name")
                    .table(Authors::Table)
                    .col(Authors::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Authors::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Authors {
    Table,
    Id,
    Name,
    Bio,
    CreatedAt,
    UpdatedAt,
}
