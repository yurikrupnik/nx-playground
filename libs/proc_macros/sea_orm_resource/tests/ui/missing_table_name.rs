//! Test that missing table_name produces a compile error

use proc_macros::SeaOrmResource;

#[derive(SeaOrmResource)]
pub struct Model {
    pub id: i32,
    pub title: String,
}

fn main() {}
