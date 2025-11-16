use serde::{Deserialize, Serialize};

/// Represents the entire database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSchema {
    pub tables: Vec<Table>,
}

impl DatabaseSchema {
    pub fn new() -> Self {
        Self { tables: Vec::new() }
    }

    pub fn add_table(&mut self, table: Table) {
        self.tables.push(table);
    }

    // pub fn get_table(&self, name: &str) -> Option<&Table> {
    //     self.tables.iter().find(|t| t.name == name)
    // }
}

/// Represents a database table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub entity_name: String,
    pub fields: Vec<Field>,
    pub relations: Vec<Relation>,
    pub indexes: Vec<Index>,
}

impl Table {
    pub fn new(name: String, entity_name: String) -> Self {
        Self {
            name,
            entity_name,
            fields: Vec::new(),
            relations: Vec::new(),
            indexes: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    // pub fn add_index(&mut self, index: Index) {
    //     self.indexes.push(index);
    // }
    //
    // pub fn get_primary_key(&self) -> Option<&Field> {
    //     self.fields.iter().find(|f| f.is_primary_key)
    // }
}

/// Represents a field/column in a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
}

impl Field {
    pub fn new(name: String, field_type: String) -> Self {
        Self {
            name,
            field_type,
            is_primary_key: false,
            is_nullable: false,
            is_unique: false,
            default_value: None,
            comment: None,
        }
    }

    /// Convert Rust type to SQL type for diagram display
    pub fn sql_type(&self) -> String {
        match self.field_type.as_str() {
            "Uuid" | "uuid::Uuid" => "uuid".to_string(),
            "String" => "varchar".to_string(),
            "i32" | "i64" | "u32" | "u64" => "integer".to_string(),
            "f32" | "f64" => "float".to_string(),
            "bool" => "boolean".to_string(),
            "DateTime<Utc>" | "chrono::DateTime<Utc>" => "timestamptz".to_string(),
            "DateTimeWithTimeZone" => "timestamptz".to_string(),
            "NaiveDate" | "chrono::NaiveDate" => "date".to_string(),
            "NaiveDateTime" | "chrono::NaiveDateTime" => "timestamp".to_string(),
            other => {
                // Handle Option<T>
                if other.starts_with("Option<") {
                    let inner = other.trim_start_matches("Option<").trim_end_matches('>');
                    return Self::new("".to_string(), inner.to_string()).sql_type();
                }
                other.to_lowercase()
            }
        }
    }
}

/// Represents a relationship between tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub relation_type: RelationType,
    pub target_table: String,
    pub from_column: Option<String>,
    pub to_column: Option<String>,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    HasMany,
    HasOne,
    BelongsTo,
    ManyToMany,
}

impl RelationType {
    pub fn to_cardinality(&self) -> &str {
        match self {
            RelationType::HasMany => "||--o{",
            RelationType::HasOne => "||--||",
            RelationType::BelongsTo => "}o--||",
            RelationType::ManyToMany => "}o--o{",
        }
    }
}

/// Represents an index on a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}
