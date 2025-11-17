use proc_macros::ApiResource;

#[derive(ApiResource)]
#[api_resource(url = true)]  // Should fail - url must be a string
pub struct User {
    id: String,
}

fn main() {}
