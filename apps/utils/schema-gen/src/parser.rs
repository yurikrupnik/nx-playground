use crate::schema::{DatabaseSchema, Field, Relation, RelationType, Table};
use std::fs;
use std::path::Path;
use syn::{Attribute, File, Item, ItemStruct, Meta, Type};
use walkdir::WalkDir;

pub struct EntityParser {
    entities_path: String,
}

impl EntityParser {
    pub fn new(entities_path: String) -> Self {
        Self { entities_path }
    }

    /// Parse all entity files in the entities directory
    pub fn parse(&self) -> color_eyre::Result<DatabaseSchema> {
        let mut schema = DatabaseSchema::new();

        // Walk through all .rs files in the entities directory
        for entry in WalkDir::new(&self.entities_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|s| s.to_str()) == Some("rs")
                    && !e.path().ends_with("mod.rs")
                    && !e.path().ends_with("prelude.rs")
            })
        {
            let path = entry.path();
            if let Ok(table) = self.parse_entity_file(path) {
                schema.add_table(table);
            }
        }

        Ok(schema)
    }

    /// Parse a single entity file
    fn parse_entity_file(&self, path: &Path) -> color_eyre::Result<Table> {
        let content = fs::read_to_string(path)?;
        let syntax_tree: File = syn::parse_file(&content)?;

        // Find the struct with #[derive(DeriveEntityModel)]
        for item in syntax_tree.items {
            if let Item::Struct(item_struct) = item {
                if self.is_entity_model(&item_struct) {
                    return self.parse_entity_struct(&item_struct);
                }
            }
        }

        Err(color_eyre::eyre::eyre!(
            "No entity model found in {:?}",
            path
        ))
    }

    /// Check if a struct is a SeaORM entity model
    fn is_entity_model(&self, item_struct: &ItemStruct) -> bool {
        item_struct.attrs.iter().any(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                return meta_list.path.is_ident("derive")
                    && meta_list.tokens.to_string().contains("DeriveEntityModel");
            }
            false
        })
    }

    /// Parse an entity struct into a Table
    fn parse_entity_struct(&self, item_struct: &ItemStruct) -> color_eyre::Result<Table> {
        let entity_name = item_struct.ident.to_string();

        // Get table name from #[sea_orm(table_name = "...")] or derive from struct name
        let table_name = self
            .extract_table_name(&item_struct.attrs)
            .unwrap_or_else(|| pluralize(&entity_name.to_lowercase()));

        let mut table = Table::new(table_name, entity_name);

        // Parse fields
        if let syn::Fields::Named(fields) = &item_struct.fields {
            for field in &fields.named {
                if let Some(ident) = &field.ident {
                    let field_name = ident.to_string();
                    let field_type = self.type_to_string(&field.ty);

                    let mut db_field = Field::new(field_name.clone(), field_type);

                    // Parse field attributes
                    for attr in &field.attrs {
                        if attr.path().is_ident("sea_orm") {
                            self.parse_field_attributes(attr, &mut db_field);
                        }
                    }

                    // Detect nullable from Option<T>
                    if self.is_option_type(&field.ty) {
                        db_field.is_nullable = true;
                    }

                    table.add_field(db_field);
                }
            }
        }

        Ok(table)
    }

    /// Extract table name from sea_orm attributes
    fn extract_table_name(&self, attrs: &[Attribute]) -> Option<String> {
        for attr in attrs {
            if let Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("sea_orm") {
                    // Parse the tokens to find table_name = "..."
                    let tokens_str = meta_list.tokens.to_string();
                    if tokens_str.contains("table_name") {
                        // Simple parsing: look for table_name = "value"
                        if let Some(start) = tokens_str.find("table_name") {
                            let after = &tokens_str[start..];
                            if let Some(quote_start) = after.find('"') {
                                if let Some(quote_end) = after[quote_start + 1..].find('"') {
                                    let value =
                                        &after[quote_start + 1..quote_start + 1 + quote_end];
                                    return Some(value.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Parse field attributes like #[sea_orm(primary_key)]
    fn parse_field_attributes(&self, attr: &Attribute, field: &mut Field) {
        if let Meta::List(meta_list) = &attr.meta {
            let tokens_str = meta_list.tokens.to_string();

            if tokens_str.contains("primary_key") {
                field.is_primary_key = true;
            }
            if tokens_str.contains("unique") {
                field.is_unique = true;
            }
            if tokens_str.contains("default_value") {
                // Extract default value if present
                if let Some(start) = tokens_str.find("default_value") {
                    let after = &tokens_str[start..];
                    if let Some(quote_start) = after.find('"') {
                        if let Some(quote_end) = after[quote_start + 1..].find('"') {
                            let value = &after[quote_start + 1..quote_start + 1 + quote_end];
                            field.default_value = Some(value.to_string());
                        }
                    }
                }
            }
        }
    }

    /// Convert Syn Type to string
    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Path(type_path) => {
                let segments: Vec<String> = type_path
                    .path
                    .segments
                    .iter()
                    .map(|seg| {
                        let ident = seg.ident.to_string();
                        if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                            let inner: Vec<String> = args
                                .args
                                .iter()
                                .map(|arg| {
                                    if let syn::GenericArgument::Type(inner_ty) = arg {
                                        self.type_to_string(inner_ty)
                                    } else {
                                        String::new()
                                    }
                                })
                                .collect();
                            format!("{}<{}>", ident, inner.join(", "))
                        } else {
                            ident
                        }
                    })
                    .collect();
                segments.join("::")
            }
            _ => "unknown".to_string(),
        }
    }

    /// Check if type is Option<T>
    fn is_option_type(&self, ty: &Type) -> bool {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "Option";
            }
        }
        false
    }
}

/// Simple pluralization (can be enhanced with pluralizer crate later)
fn pluralize(word: &str) -> String {
    if word.ends_with('y') {
        format!("{}ies", &word[..word.len() - 1])
    } else if word.ends_with('s') {
        word.to_string()
    } else {
        format!("{}s", word)
    }
}

/// Parse relations from the Relation enum in entity files
pub fn parse_relations(entities_path: &str) -> color_eyre::Result<Vec<(String, Vec<Relation>)>> {
    let mut relations_map = Vec::new();

    for entry in WalkDir::new(entities_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("rs")
                && !e.path().ends_with("mod.rs")
                && !e.path().ends_with("prelude.rs")
        })
    {
        let path = entry.path();
        let content = fs::read_to_string(path)?;
        let syntax_tree: File = syn::parse_file(&content)?;

        let mut entity_name = String::new();
        let mut relations = Vec::new();

        // Extract entity name from Model struct
        for item in &syntax_tree.items {
            if let Item::Struct(item_struct) = item {
                if item_struct.ident == "Model" {
                    // Get table name from attributes
                    for attr in &item_struct.attrs {
                        if let Meta::List(meta_list) = &attr.meta {
                            if meta_list.path.is_ident("sea_orm") {
                                let tokens_str = meta_list.tokens.to_string();
                                if let Some(start) = tokens_str.find("table_name") {
                                    let after = &tokens_str[start..];
                                    if let Some(quote_start) = after.find('"') {
                                        if let Some(quote_end) = after[quote_start + 1..].find('"')
                                        {
                                            entity_name = after
                                                [quote_start + 1..quote_start + 1 + quote_end]
                                                .to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse Relation enum
        for item in &syntax_tree.items {
            if let Item::Enum(item_enum) = item {
                if item_enum.ident == "Relation" {
                    for variant in &item_enum.variants {
                        for attr in &variant.attrs {
                            if attr.path().is_ident("sea_orm") {
                                if let Some(relation) = parse_relation_attribute(attr) {
                                    relations.push(relation);
                                }
                            }
                        }
                    }
                }
            }
        }

        if !entity_name.is_empty() && !relations.is_empty() {
            relations_map.push((entity_name, relations));
        }
    }

    Ok(relations_map)
}

fn parse_relation_attribute(attr: &Attribute) -> Option<Relation> {
    if let Meta::List(meta_list) = &attr.meta {
        let tokens_str = meta_list.tokens.to_string();
        let mut relation_type = None;
        let mut target_table = String::new();
        let mut from_column = None;
        let mut to_column = None;
        let on_delete = None;
        let on_update = None;

        // Determine relation type and extract target entity
        if let Some(start) = tokens_str.find("has_many") {
            relation_type = Some(RelationType::HasMany);
            target_table = extract_target_entity(&tokens_str[start..]);
        } else if let Some(start) = tokens_str.find("has_one") {
            relation_type = Some(RelationType::HasOne);
            target_table = extract_target_entity(&tokens_str[start..]);
        } else if let Some(start) = tokens_str.find("belongs_to") {
            relation_type = Some(RelationType::BelongsTo);
            target_table = extract_target_entity(&tokens_str[start..]);
        }

        // Extract from column
        if let Some(from_start) = tokens_str.find("from =") {
            from_column = extract_column_name(&tokens_str[from_start..]);
        }

        // Extract to column
        if let Some(to_start) = tokens_str.find("to =") {
            to_column = extract_column_name(&tokens_str[to_start..]);
        }

        if let Some(rt) = relation_type {
            return Some(Relation {
                relation_type: rt,
                target_table,
                from_column,
                to_column,
                on_delete,
                on_update,
            });
        }
    }
    None
}

/// Extract target entity name from relation attribute
/// Example: `has_many = "super::book::Entity"` -> "books"
fn extract_target_entity(text: &str) -> String {
    if let Some(quote_start) = text.find('"') {
        if let Some(quote_end) = text[quote_start + 1..].find('"') {
            let entity_path = &text[quote_start + 1..quote_start + 1 + quote_end];
            // Split by "::" and find the entity name (second to last segment)
            // Example: "super::book::Entity" -> ["super", "book", "Entity"]
            let parts: Vec<&str> = entity_path.split("::").collect();
            if parts.len() >= 2 {
                // Get the second-to-last part (the entity name, not "Entity")
                let entity_name = parts[parts.len() - 2];
                // Convert to lowercase and pluralize
                return pluralize(&entity_name.to_lowercase());
            }
        }
    }
    String::new()
}

/// Extract column name from attribute like `from = "Column::AuthorId"` or `to = "super::author::Column::Id"`
fn extract_column_name(text: &str) -> Option<String> {
    if let Some(quote_start) = text.find('"') {
        if let Some(quote_end) = text[quote_start + 1..].find('"') {
            let column_path = &text[quote_start + 1..quote_start + 1 + quote_end];

            // For paths like "super::author::Column::Id", split and find "Column::"
            if column_path.contains("Column::") {
                if let Some(column_start) = column_path.find("Column::") {
                    let after_column = &column_path[column_start + 8..]; // Skip "Column::"
                                                                         // Get everything after "Column::" (the actual column name)
                    let column_name = after_column.trim();
                    return Some(to_snake_case(column_name));
                }
            }
        }
    }
    None
}

/// Convert CamelCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap());
    }
    result
}
