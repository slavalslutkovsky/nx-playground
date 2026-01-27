use crate::generators::DiagramGenerator;
use crate::schema::{DatabaseSchema, Field, Index, RelationType, Table};

/// Generates Atlas HCL schema format
/// Docs: https://atlasgo.io/atlas-schema/hcl
pub struct AtlasGenerator;

impl AtlasGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Convert Rust/SQL type to Atlas HCL type
    fn atlas_type(&self, field: &Field) -> String {
        let sql_type = field.sql_type();
        match sql_type.as_str() {
            "uuid" => "uuid".to_string(),
            "varchar" => "varchar(255)".to_string(),
            "text" => "text".to_string(),
            "integer" => "integer".to_string(),
            "bigint" => "bigint".to_string(),
            "smallint" => "smallint".to_string(),
            "float" => "real".to_string(),
            "double" => "double precision".to_string(),
            "boolean" => "boolean".to_string(),
            "timestamptz" => "timestamptz".to_string(),
            "timestamp" => "timestamp".to_string(),
            "date" => "date".to_string(),
            "time" => "time".to_string(),
            "jsonb" => "jsonb".to_string(),
            "json" => "json".to_string(),
            "bytea" => "bytea".to_string(),
            other => other.to_string(),
        }
    }

    /// Generate a column definition
    fn generate_column(&self, field: &Field, indent: &str) -> String {
        let mut parts = vec![format!("{}column \"{}\" {{", indent, field.name)];
        let inner_indent = format!("{}  ", indent);

        // Type
        parts.push(format!(
            "{}type = sql(\"{}\")",
            inner_indent,
            self.atlas_type(field)
        ));

        // Nullable
        if !field.is_nullable && !field.is_primary_key {
            parts.push(format!("{}null = false", inner_indent));
        }

        // Default value
        if let Some(default) = &field.default_value {
            let default_expr = match default.as_str() {
                "gen_random_uuid()" => "sql(\"gen_random_uuid()\")",
                "now()" | "CURRENT_TIMESTAMP" => "sql(\"now()\")",
                "true" => "true",
                "false" => "false",
                d if d.starts_with('\'') => default,
                d => d,
            };
            parts.push(format!("{}default = {}", inner_indent, default_expr));
        }

        // Comment
        if let Some(comment) = &field.comment {
            parts.push(format!(
                "{}comment = \"{}\"",
                inner_indent,
                comment.replace('"', "\\\"")
            ));
        }

        parts.push(format!("{}}}", indent));
        parts.join("\n")
    }

    /// Generate primary key constraint
    fn generate_primary_key(&self, table: &Table, indent: &str) -> Option<String> {
        let pk_columns: Vec<&str> = table
            .fields
            .iter()
            .filter(|f| f.is_primary_key)
            .map(|f| f.name.as_str())
            .collect();

        if pk_columns.is_empty() {
            return None;
        }

        let columns = pk_columns
            .iter()
            .map(|c| format!("column.{}", c))
            .collect::<Vec<_>>()
            .join(", ");

        Some(format!(
            "{}primary_key {{\n{}  columns = [{}]\n{}}}",
            indent, indent, columns, indent
        ))
    }

    /// Generate unique constraints
    fn generate_unique_constraints(&self, table: &Table, indent: &str) -> Vec<String> {
        table
            .fields
            .iter()
            .filter(|f| f.is_unique && !f.is_primary_key)
            .map(|f| {
                format!(
                    "{}unique \"{}_{}_key\" {{\n{}  columns = [column.{}]\n{}}}",
                    indent, table.name, f.name, indent, f.name, indent
                )
            })
            .collect()
    }

    /// Generate index definitions
    fn generate_indexes(&self, table: &Table, indent: &str) -> Vec<String> {
        table
            .indexes
            .iter()
            .map(|idx| self.generate_index(table, idx, indent))
            .collect()
    }

    fn generate_index(&self, _table: &Table, index: &Index, indent: &str) -> String {
        let columns = index
            .columns
            .iter()
            .map(|c| format!("column.{}", c))
            .collect::<Vec<_>>()
            .join(", ");

        let mut parts = vec![format!("{}index \"{}\" {{", indent, index.name)];
        let inner_indent = format!("{}  ", indent);

        parts.push(format!("{}columns = [{}]", inner_indent, columns));

        if index.is_unique {
            parts.push(format!("{}unique = true", inner_indent));
        }

        parts.push(format!("{}}}", indent));
        parts.join("\n")
    }

    /// Generate foreign key definitions
    fn generate_foreign_keys(&self, table: &Table, indent: &str) -> Vec<String> {
        table
            .relations
            .iter()
            .filter(|r| r.relation_type == RelationType::BelongsTo)
            .filter_map(|r| {
                let from_col = r.from_column.as_deref()?;
                let to_col = r.to_column.as_deref().unwrap_or("id");

                let mut parts = vec![format!(
                    "{}foreign_key \"fk_{}_{}_{}\" {{",
                    indent, table.name, r.target_table, from_col
                )];
                let inner_indent = format!("{}  ", indent);

                parts.push(format!(
                    "{}columns     = [column.{}]",
                    inner_indent, from_col
                ));
                parts.push(format!(
                    "{}ref_columns = [table.{}.column.{}]",
                    inner_indent, r.target_table, to_col
                ));

                if let Some(on_delete) = &r.on_delete {
                    parts.push(format!(
                        "{}on_delete   = {}",
                        inner_indent,
                        on_delete.to_uppercase()
                    ));
                }
                if let Some(on_update) = &r.on_update {
                    parts.push(format!(
                        "{}on_update   = {}",
                        inner_indent,
                        on_update.to_uppercase()
                    ));
                }

                parts.push(format!("{}}}", indent));
                Some(parts.join("\n"))
            })
            .collect()
    }

    /// Generate a complete table definition
    fn generate_table(&self, table: &Table) -> String {
        let mut parts = vec![format!("table \"{}\" {{", table.name)];
        parts.push("  schema = schema.public".to_string());
        parts.push(String::new());

        // Columns
        for field in &table.fields {
            parts.push(self.generate_column(field, "  "));
        }

        // Primary key
        if let Some(pk) = self.generate_primary_key(table, "  ") {
            parts.push(String::new());
            parts.push(pk);
        }

        // Unique constraints
        let uniques = self.generate_unique_constraints(table, "  ");
        if !uniques.is_empty() {
            parts.push(String::new());
            parts.extend(uniques);
        }

        // Indexes
        let indexes = self.generate_indexes(table, "  ");
        if !indexes.is_empty() {
            parts.push(String::new());
            parts.extend(indexes);
        }

        // Foreign keys
        let fks = self.generate_foreign_keys(table, "  ");
        if !fks.is_empty() {
            parts.push(String::new());
            parts.extend(fks);
        }

        parts.push("}".to_string());
        parts.join("\n")
    }
}

impl DiagramGenerator for AtlasGenerator {
    fn generate(&self, schema: &DatabaseSchema) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "# Atlas HCL Schema\n# Auto-generated from SeaORM entities on {}\n# Docs: https://atlasgo.io/atlas-schema/hcl\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Schema definition
        output.push_str("schema \"public\" {\n");
        output.push_str("  comment = \"Zerg application schema\"\n");
        output.push_str("}\n\n");

        // Extensions
        output.push_str("# PostgreSQL Extensions\n");
        output.push_str("extension \"pgcrypto\" {\n");
        output.push_str("  schema  = schema.public\n");
        output.push_str("  version = \"1.3\"\n");
        output.push_str("}\n\n");
        output.push_str("extension \"uuid-ossp\" {\n");
        output.push_str("  schema  = schema.public\n");
        output.push_str("  version = \"1.1\"\n");
        output.push_str("}\n\n");

        // Tables
        for table in &schema.tables {
            output.push_str(&self.generate_table(table));
            output.push_str("\n\n");
        }

        output
    }
}

impl Default for AtlasGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Field, Relation, RelationType, Table};

    #[test]
    fn test_atlas_type_conversion() {
        let gen = AtlasGenerator::new();

        let uuid_field = Field::new("id".to_string(), "Uuid".to_string());
        assert_eq!(gen.atlas_type(&uuid_field), "uuid");

        let string_field = Field::new("name".to_string(), "String".to_string());
        assert_eq!(gen.atlas_type(&string_field), "varchar(255)");
    }

    #[test]
    fn test_generate_column() {
        let gen = AtlasGenerator::new();

        let mut field = Field::new("email".to_string(), "String".to_string());
        field.is_unique = true;
        field.is_nullable = false;

        let output = gen.generate_column(&field, "  ");
        assert!(output.contains("column \"email\""));
        assert!(output.contains("null = false"));
    }
}
