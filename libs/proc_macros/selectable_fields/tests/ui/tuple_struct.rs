use selectable_fields::SelectableFields;

#[derive(SelectableFields)]
pub struct Wrapper(String, i32);  // Tuple struct - only named fields supported

fn main() {}
