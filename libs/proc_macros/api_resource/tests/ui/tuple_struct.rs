use proc_macros::ApiResource;

#[derive(ApiResource)]
#[api_resource(collection = 123)]  // Should fail - collection must be a string
pub struct User {
    id: String,
}

fn main () {}
