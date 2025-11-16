use selectable_fields::SelectableFields;

#[derive(SelectableFields)]
pub struct Empty;  // Unit struct - should handle gracefully or fail

fn main() {}
