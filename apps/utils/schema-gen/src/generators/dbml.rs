use crate::generators::DiagramGenerator;
use crate::schema::{DatabaseSchema, Field, RelationType, Table};

pub struct DbmlGenerator;

impl DbmlGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_field_line(&self, field: &Field) -> String {
        let sql_type = field.sql_type();
        let mut attributes = Vec::new();

        if field.is_primary_key {
            attributes.push("pk".to_string());
        }
        if field.is_unique {
            attributes.push("unique".to_string());
        }
        if !field.is_nullable && !field.is_primary_key {
            attributes.push("not null".to_string());
        }
        if let Some(default) = &field.default_value {
            attributes.push(format!("default: `{}`", default));
        }
        if let Some(comment) = &field.comment {
            attributes.push(format!("note: '{}'", comment));
        }

        let attr_str = if !attributes.is_empty() {
            format!(" [{}]", attributes.join(", "))
        } else {
            String::new()
        };

        format!("  {} {}{}", field.name, sql_type, attr_str)
    }

    fn generate_table(&self, table: &Table) -> String {
        let mut output = String::new();

        output.push_str(&format!("Table {} {{\n", table.name));

        // Fields
        for field in &table.fields {
            output.push_str(&self.generate_field_line(field));
            output.push('\n');
        }

        // Indexes
        if !table.indexes.is_empty() {
            output.push_str("\n  indexes {\n");
            for index in &table.indexes {
                let columns = index.columns.join(", ");
                let index_type = if index.is_unique { "unique, " } else { "" };
                output.push_str(&format!(
                    "    ({}) [{}name: '{}']\n",
                    columns, index_type, index.name
                ));
            }
            output.push_str("  }\n");
        }

        output.push_str("}\n");
        output
    }

    fn generate_relationships(&self, schema: &DatabaseSchema) -> String {
        let mut output = String::new();
        let mut seen_relations = std::collections::HashSet::new();

        for table in &schema.tables {
            for relation in &table.relations {
                let relation_key = format!(
                    "{}_{}_{:?}",
                    table.name, relation.target_table, relation.relation_type
                );

                if !seen_relations.contains(&relation_key) {
                    seen_relations.insert(relation_key);

                    // For belongs_to, from_column is the FK in current table, to_column is PK in target table
                    let from_col = relation.from_column.as_deref().unwrap_or("id");
                    let to_col = relation.to_column.as_deref().unwrap_or("id");

                    match relation.relation_type {
                        RelationType::BelongsTo => {
                            // Many-to-one relationship
                            let mut ref_attrs = Vec::new();
                            if let Some(on_delete) = &relation.on_delete {
                                ref_attrs.push(format!("delete: {}", on_delete.to_lowercase()));
                            }
                            if let Some(on_update) = &relation.on_update {
                                ref_attrs.push(format!("update: {}", on_update.to_lowercase()));
                            }

                            let attrs = if !ref_attrs.is_empty() {
                                format!(" [{}]", ref_attrs.join(", "))
                            } else {
                                String::new()
                            };

                            output.push_str(&format!(
                                "Ref: {}.{} > {}.{}{}\n",
                                table.name, from_col, relation.target_table, to_col, attrs
                            ));
                        }
                        RelationType::HasMany => {
                            // This is the inverse of BelongsTo, will be captured from the other side
                        }
                        RelationType::HasOne => {
                            output.push_str(&format!(
                                "Ref: {}.{} - {}.{}\n",
                                table.name, from_col, relation.target_table, to_col
                            ));
                        }
                        RelationType::ManyToMany => {
                            output.push_str(&format!(
                                "Ref: {}.{} <> {}.{}\n",
                                table.name, from_col, relation.target_table, to_col
                            ));
                        }
                    }
                }
            }
        }

        output
    }
}

impl DiagramGenerator for DbmlGenerator {
    fn generate(&self, schema: &DatabaseSchema) -> String {
        let mut output = String::new();

        // Header comment
        output.push_str(&format!(
            "// Database Schema\n// Auto-generated from SeaORM entities on {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Generate all tables
        for table in &schema.tables {
            output.push_str(&self.generate_table(table));
            output.push('\n');
        }

        // Generate relationships
        output.push_str(&self.generate_relationships(schema));

        output
    }
}

impl Default for DbmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}
