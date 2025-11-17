use proc_macros::SelectableFields;

#[derive(SelectableFields)]
pub struct Empty;  // Unit struct - should handle gracefully or fail

fn main() {}
