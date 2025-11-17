use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Books::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Books::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Books::Title).string().not_null())
                    .col(ColumnDef::new(Books::Description).text())
                    .col(ColumnDef::new(Books::AuthorId).uuid().not_null())
                    .col(ColumnDef::new(Books::PublishedDate).date())
                    .col(ColumnDef::new(Books::Isbn).string())
                    .col(
                        ColumnDef::new(Books::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Books::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_books_author")
                            .from(Books::Table, Books::AuthorId)
                            .to(Authors::Table, Authors::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_books_author_id")
                    .table(Books::Table)
                    .col(Books::AuthorId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_books_title")
                    .table(Books::Table)
                    .col(Books::Title)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_books_isbn")
                    .table(Books::Table)
                    .col(Books::Isbn)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Books::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Books {
    Table,
    Id,
    Title,
    Description,
    AuthorId,
    PublishedDate,
    Isbn,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Authors {
    Table,
    Id,
}
