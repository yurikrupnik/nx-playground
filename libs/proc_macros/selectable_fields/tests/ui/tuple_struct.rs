use proc_macros::SelectableFields;

#[derive(SelectableFields)]
pub struct Wrapper(String, i32);  // Tuple struct - only named fields supported

fn main() {}
