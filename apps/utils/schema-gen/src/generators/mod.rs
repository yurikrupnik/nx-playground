pub mod dbml;
pub mod mermaid;

pub use dbml::DbmlGenerator;
pub use mermaid::MermaidGenerator;

use crate::schema::DatabaseSchema;

pub trait DiagramGenerator {
    fn generate(&self, schema: &DatabaseSchema) -> String;
}
