// Integration tests for ApiResource derive macro
// Import the derive macro and trait from proc_macros crate (not directly from api_resource)
use proc_macros::ApiResource;

#[derive(ApiResource)]
#[allow(dead_code)]
pub struct User {
    id: String,
    email: String,
}

#[derive(ApiResource)]
#[api_resource(collection = "people", url = "/api/users", tag = "User Management")]
#[allow(dead_code)]
pub struct Person {
    id: String,
}

#[derive(ApiResource)]
#[allow(dead_code)]
pub struct Story {
    id: String,
}

#[test]
fn test_basic_user() {
    assert_eq!(User::COLLECTION, "users");
    assert_eq!(User::URL, "/user");
    assert_eq!(User::URL_WITH_ID, "/user/{id}");
    assert_eq!(User::TAG, "Users");
}

#[test]
fn test_custom_attributes() {
    assert_eq!(Person::COLLECTION, "people");
    assert_eq!(Person::URL, "/api/users");
    assert_eq!(Person::URL_WITH_ID, "/api/users/{id}");
    assert_eq!(Person::TAG, "User Management");
}

#[test]
fn test_irregular_pluralization() {
    assert_eq!(Story::COLLECTION, "stories");
    assert_eq!(Story::URL, "/story");
    assert_eq!(Story::TAG, "Stories");
}
