use utoipa::OpenApi;

use crate::{
    dto::{
        author::{AuthorResponseDto, CreateAuthorDto, UpdateAuthorDto},
        book::{BookResponseDto, BookWithAuthorDto, CreateBookDto, UpdateBookDto},
        todo::{CreateTodoDto, TodoResponseDto, UpdateTodoDto},
        user::{CreateUserDto, UpdateUserDto, UserResponseDto},
    },
    handlers::fields::{FieldInfo, FieldsResponse, ResourceFields},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Zerg API",
        version = "1.0.0",
        description = "A modern REST API built with Axum and SeaORM",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    paths(
        // Author endpoints
        crate::handlers::authors::create_author,
        crate::handlers::authors::list_authors,
        crate::handlers::authors::get_author,
        crate::handlers::authors::update_author,
        crate::handlers::authors::delete_author,
        // Book endpoints
        crate::handlers::books::create_book,
        crate::handlers::books::list_books,
        crate::handlers::books::list_books_with_authors,
        crate::handlers::books::get_book,
        crate::handlers::books::get_book_with_author,
        crate::handlers::books::update_book,
        crate::handlers::books::delete_book,
        // Todo endpoints
        crate::handlers::todos::create_todo,
        crate::handlers::todos::list_todos,
        crate::handlers::todos::get_todo,
        crate::handlers::todos::update_todo,
        crate::handlers::todos::delete_todo,
        crate::handlers::fields::list_todo_fields,
        // User endpoints
        crate::handlers::users::create_user,
        crate::handlers::users::list_users,
        crate::handlers::users::get_user,
        crate::handlers::users::update_user,
        crate::handlers::users::delete_user,
        crate::handlers::fields::list_user_fields,
        // Meta endpoints
        crate::handlers::fields::list_all_fields,
    ),
    components(
        schemas(
            // Author schemas
            CreateAuthorDto,
            UpdateAuthorDto,
            AuthorResponseDto,
            // Book schemas
            CreateBookDto,
            UpdateBookDto,
            BookResponseDto,
            BookWithAuthorDto,
            // Todo schemas
            CreateTodoDto,
            UpdateTodoDto,
            TodoResponseDto,
            // User schemas
            CreateUserDto,
            UpdateUserDto,
            UserResponseDto,
            // Field metadata schemas
            FieldInfo,
            ResourceFields,
            FieldsResponse,
        )
    ),
    tags(
        (name = "authors", description = "Author management endpoints"),
        (name = "books", description = "Book management endpoints (with FK relationships)"),
        (name = "todos", description = "Todo management endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "meta", description = "API metadata and introspection endpoints")
    )
)]
pub struct ApiDoc;
