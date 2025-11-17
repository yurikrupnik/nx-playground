use proc_macros::ApiResource;

#[derive(ApiResource)]
#[api_resource(unknown_attr = "value")]  // Invalid attribute should fail
pub struct User {
    id: String,
}

fn main() {}
