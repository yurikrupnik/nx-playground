use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Seed Users
        db.execute_unprepared(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
            VALUES
                ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'admin', 'admin@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.V5ferDZKUqIW6K', NOW(), NOW()),
                ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'john_doe', 'john@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.V5ferDZKUqIW6K', NOW(), NOW()),
                ('c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'jane_smith', 'jane@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.V5ferDZKUqIW6K', NOW(), NOW())
            ON CONFLICT DO NOTHING
            "#,
        )
        .await?;

        // Seed Todos
        db.execute_unprepared(
            r#"
            INSERT INTO todos (id, name, description, completed, created_at, updated_at)
            VALUES
                ('d3eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'Setup development environment', 'Install all required tools and dependencies', true, NOW(), NOW()),
                ('e4eebc99-9c0b-4ef8-bb6d-6bb9bd380a55', 'Write unit tests', 'Add comprehensive test coverage for core modules', false, NOW(), NOW()),
                ('f5eebc99-9c0b-4ef8-bb6d-6bb9bd380a66', 'Review pull requests', 'Review pending PRs from team members', false, NOW(), NOW()),
                ('06eebc99-9c0b-4ef8-bb6d-6bb9bd380a77', 'Update documentation', 'Update API docs with new endpoints', false, NOW(), NOW()),
                ('17eebc99-9c0b-4ef8-bb6d-6bb9bd380a88', 'Deploy to staging', 'Deploy latest changes to staging environment', false, NOW(), NOW())
            ON CONFLICT DO NOTHING
            "#,
        )
        .await?;

        // Seed Authors
        db.execute_unprepared(
            r#"
            INSERT INTO authors (id, name, bio, created_at, updated_at)
            VALUES
                ('28eebc99-9c0b-4ef8-bb6d-6bb9bd380a99', 'George Orwell', 'English novelist and essayist, known for his dystopian works.', NOW(), NOW()),
                ('39eebc99-9c0b-4ef8-bb6d-6bb9bd380aaa', 'Jane Austen', 'English novelist known for her witty social commentary.', NOW(), NOW()),
                ('4aeebc99-9c0b-4ef8-bb6d-6bb9bd380abb', 'Isaac Asimov', 'American writer and professor of biochemistry, known for science fiction.', NOW(), NOW())
            ON CONFLICT DO NOTHING
            "#,
        )
        .await?;

        // Seed Books (must come after Authors due to a foreign key)
        db.execute_unprepared(
            r#"
            INSERT INTO books (id, title, description, author_id, published_date, isbn, created_at, updated_at)
            VALUES
                ('5beebc99-9c0b-4ef8-bb6d-6bb9bd380acc', '1984', 'A dystopian social science fiction novel.', '28eebc99-9c0b-4ef8-bb6d-6bb9bd380a99', '1949-06-08', '978-0451524935', NOW(), NOW()),
                ('6ceebc99-9c0b-4ef8-bb6d-6bb9bd380add', 'Animal Farm', 'A satirical allegorical novella.', '28eebc99-9c0b-4ef8-bb6d-6bb9bd380a99', '1945-08-17', '978-0451526342', NOW(), NOW()),
                ('7deebc99-9c0b-4ef8-bb6d-6bb9bd380aee', 'Pride and Prejudice', 'A romantic novel of manners.', '39eebc99-9c0b-4ef8-bb6d-6bb9bd380aaa', '1813-01-28', '978-0141439518', NOW(), NOW()),
                ('8eeebc99-9c0b-4ef8-bb6d-6bb9bd380aff', 'Foundation', 'The first novel in the Foundation series.', '4aeebc99-9c0b-4ef8-bb6d-6bb9bd380abb', '1951-06-01', '978-0553293357', NOW(), NOW()),
                ('9feebc99-9c0b-4ef8-bb6d-6bb9bd380b00', 'I, Robot', 'A collection of nine science fiction short stories.', '4aeebc99-9c0b-4ef8-bb6d-6bb9bd380abb', '1950-12-02', '978-0553382563', NOW(), NOW())
            ON CONFLICT DO NOTHING
            "#,
        )
        .await?;

        // Seed Projects
        db.execute_unprepared(
            r#"
            INSERT INTO projects (title, description, completed, created_at, updated_at)
            VALUES
                ('Website Redesign', 'Complete overhaul of company website with modern UI/UX', false, NOW(), NOW()),
                ('Mobile App MVP', 'Develop minimum viable product for iOS and Android', false, NOW(), NOW()),
                ('API Integration', 'Integrate third-party payment gateway', true, NOW(), NOW()),
                ('Database Migration', 'Migrate from MySQL to PostgreSQL', true, NOW(), NOW()),
                ('CI/CD Pipeline', 'Setup automated testing and deployment pipeline', false, NOW(), NOW())
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Delete it in reverse order due to foreign keys
        db.execute_unprepared(
            r#"
            DELETE FROM books WHERE id IN (
                '5beebc99-9c0b-4ef8-bb6d-6bb9bd380acc',
                '6ceebc99-9c0b-4ef8-bb6d-6bb9bd380add',
                '7deebc99-9c0b-4ef8-bb6d-6bb9bd380aee',
                '8eeebc99-9c0b-4ef8-bb6d-6bb9bd380aff',
                '9feebc99-9c0b-4ef8-bb6d-6bb9bd380b00'
            )
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            DELETE FROM authors WHERE id IN (
                '28eebc99-9c0b-4ef8-bb6d-6bb9bd380a99',
                '39eebc99-9c0b-4ef8-bb6d-6bb9bd380aaa',
                '4aeebc99-9c0b-4ef8-bb6d-6bb9bd380abb'
            )
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            DELETE FROM todos WHERE id IN (
                'd3eebc99-9c0b-4ef8-bb6d-6bb9bd380a44',
                'e4eebc99-9c0b-4ef8-bb6d-6bb9bd380a55',
                'f5eebc99-9c0b-4ef8-bb6d-6bb9bd380a66',
                '06eebc99-9c0b-4ef8-bb6d-6bb9bd380a77',
                '17eebc99-9c0b-4ef8-bb6d-6bb9bd380a88'
            )
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            DELETE FROM users WHERE id IN (
                'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
                'b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22',
                'c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a33'
            )
            "#,
        )
        .await?;

        // Projects use auto-increment, delete it by title
        db.execute_unprepared(
            r#"
            DELETE FROM projects WHERE title IN (
                'Website Redesign',
                'Mobile App MVP',
                'API Integration',
                'Database Migration',
                'CI/CD Pipeline'
            )
            "#,
        )
        .await?;

        Ok(())
    }
}
