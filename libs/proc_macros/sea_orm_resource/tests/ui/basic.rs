//! Test that basic usage compiles successfully

use proc_macros::{ApiResource, SeaOrmResource};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SeaOrmResource)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub completed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

fn main() {
    // Verify constants are accessible
    let _url = Model::URL;
    let _url_with_id = Model::URL_WITH_ID;
    let _collection = Model::COLLECTION;
    let _tag = Model::TAG;
}
