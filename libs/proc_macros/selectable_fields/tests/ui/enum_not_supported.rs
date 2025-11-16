use selectable_fields::SelectableFields;

#[derive(SelectableFields)]
pub enum Status {  // Should panic - only structs supported
    Active,
    Inactive,
}

fn main() {}
