pub use sea_orm_migration::prelude::*;

mod m20250114_000001_create_users_table;
mod m20250114_000002_create_todos_table;
mod m20250114_000003_create_authors_table;
mod m20250114_000004_create_books_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250114_000001_create_users_table::Migration),
            Box::new(m20250114_000002_create_todos_table::Migration),
            Box::new(m20250114_000003_create_authors_table::Migration),
            Box::new(m20250114_000004_create_books_table::Migration),
        ]
    }
}
