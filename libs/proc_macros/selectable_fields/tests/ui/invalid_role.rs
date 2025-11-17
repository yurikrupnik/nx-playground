use proc_macros::SelectableFields;

#[derive(SelectableFields)]
pub struct User {
    id: String,
    #[field(role = "superuser")]  // Invalid role - should be anonymous, user, or admin
    email: String,
}

fn main() {}
