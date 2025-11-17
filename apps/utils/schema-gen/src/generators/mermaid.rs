use crate::generators::DiagramGenerator;
use crate::schema::{DatabaseSchema, Field, RelationType, Table};

pub struct MermaidGenerator;

impl MermaidGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_field_line(&self, field: &Field) -> String {
        let sql_type = field.sql_type();
        let mut constraints = Vec::new();

        if field.is_primary_key {
            constraints.push("PK".to_string());
        }
        if field.is_unique {
            constraints.push("UNIQUE".to_string());
        }
        if !field.is_nullable && !field.is_primary_key {
            constraints.push("NOT NULL".to_string());
        }
        if let Some(default) = &field.default_value {
            constraints.push(format!("DEFAULT {}", default));
        }

        let constraint_str = if !constraints.is_empty() {
            format!(" \"{}\"", constraints.join(", "))
        } else {
            String::new()
        };

        format!("        {} {}{}", sql_type, field.name, constraint_str)
    }

    fn generate_table(&self, table: &Table) -> String {
        let mut output = String::new();

        output.push_str(&format!("    {} {{\n", table.name.to_uppercase()));

        for field in &table.fields {
            output.push_str(&self.generate_field_line(field));
            output.push('\n');
        }

        output.push_str("    }\n");
        output
    }

    fn generate_relationships(&self, schema: &DatabaseSchema) -> String {
        let mut output = String::new();
        let mut seen_relations = std::collections::HashSet::new();

        for table in &schema.tables {
            for relation in &table.relations {
                // Create a unique key for this relation to avoid duplicates
                let relation_key = if relation.relation_type == RelationType::BelongsTo {
                    format!("{}_{}", table.name, relation.target_table)
                } else {
                    format!("{}_{}", relation.target_table, table.name)
                };

                if !seen_relations.contains(&relation_key) {
                    seen_relations.insert(relation_key);

                    let cardinality = relation.relation_type.to_cardinality();
                    let label = match relation.relation_type {
                        RelationType::HasMany => "has",
                        RelationType::HasOne => "has",
                        RelationType::BelongsTo => "belongs to",
                        RelationType::ManyToMany => "related to",
                    };

                    // For BelongsTo, reverse the direction
                    if relation.relation_type == RelationType::BelongsTo {
                        output.push_str(&format!(
                            "    {} ||--o{{ {} : \"{}\"\n",
                            relation.target_table.to_uppercase(),
                            table.name.to_uppercase(),
                            label
                        ));
                    } else {
                        output.push_str(&format!(
                            "    {} {} {} : \"{}\"\n",
                            table.name.to_uppercase(),
                            cardinality,
                            relation.target_table.to_uppercase(),
                            label
                        ));
                    }
                }
            }
        }

        output
    }

    fn generate_indexes_section(&self, schema: &DatabaseSchema) -> String {
        let mut output = String::new();
        let mut has_indexes = false;

        for table in &schema.tables {
            if !table.indexes.is_empty() {
                if !has_indexes {
                    output.push_str("\n## Indexes\n\n");
                    has_indexes = true;
                }
                output.push_str(&format!("- **{}**:", table.name));
                let index_names: Vec<String> =
                    table.indexes.iter().map(|i| i.name.clone()).collect();
                output.push_str(&format!(" {}\n", index_names.join(", ")));
            }
        }

        output
    }
}

impl DiagramGenerator for MermaidGenerator {
    fn generate(&self, schema: &DatabaseSchema) -> String {
        let mut output = String::new();

        // Header
        output.push_str("# Database Schema\n\n");
        output.push_str(&format!(
            "Auto-generated from SeaORM entities on {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Mermaid diagram
        output.push_str("```mermaid\n");
        output.push_str("erDiagram\n");

        // Generate relationships first (they look better at the top)
        output.push_str(&self.generate_relationships(schema));
        output.push('\n');

        // Generate tables
        for table in &schema.tables {
            output.push_str(&self.generate_table(table));
            output.push('\n');
        }

        output.push_str("```\n");

        // Add indexes section
        output.push_str(&self.generate_indexes_section(schema));

        output
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new()
    }
}
