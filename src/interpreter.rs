use crate::ast::*;
use std::collections::{HashMap, HashSet};

pub struct Interpreter {
    schema: Schema,
}

#[derive(Debug)]
pub enum ValidationError {
    DuplicateTable(String),
    DuplicateColumn { table: String, column: String },
    MultiplePrimaryKeys(String),
    TableNotFound(String),
    ColumnNotFound { table: String, column: String },
    InvalidRelationship(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateTable(name) => {
                write!(f, "Duplicate table definition: {}", name)
            }
            ValidationError::DuplicateColumn { table, column } => {
                write!(f, "Duplicate column '{}' in table '{}'", column, table)
            }
            ValidationError::MultiplePrimaryKeys(table) => {
                write!(f, "Table '{}' has multiple primary keys", table)
            }
            ValidationError::TableNotFound(table) => {
                write!(f, "Table '{}' not found", table)
            }
            ValidationError::ColumnNotFound { table, column } => {
                write!(f, "Column '{}' not found in table '{}'", column, table)
            }
            ValidationError::InvalidRelationship(msg) => {
                write!(f, "Invalid relationship: {}", msg)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl Interpreter {
    pub fn new(schema: Schema) -> Self {
        Interpreter { schema }
    }
    
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Validate tables
        if let Err(e) = self.validate_tables() {
            errors.extend(e);
        }
        
        // Validate relationships
        if let Err(e) = self.validate_relationships() {
            errors.extend(e);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    fn validate_tables(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut table_names = HashSet::new();
        
        for table in &self.schema.tables {
            // Check for duplicate tables
            if !table_names.insert(&table.name) {
                errors.push(ValidationError::DuplicateTable(table.name.clone()));
                continue;
            }
            
            // Check for duplicate columns
            let mut column_names = HashSet::new();
            for column in &table.columns {
                if !column_names.insert(&column.name) {
                    errors.push(ValidationError::DuplicateColumn {
                        table: table.name.clone(),
                        column: column.name.clone(),
                    });
                }
            }
            
            // Check for multiple primary keys
            let pk_count = table.columns.iter()
                .filter(|c| c.is_primary_key())
                .count();
            
            if pk_count > 1 {
                errors.push(ValidationError::MultiplePrimaryKeys(table.name.clone()));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    fn validate_relationships(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let table_map: HashMap<_, _> = self.schema.tables.iter()
            .map(|t| (&t.name, t))
            .collect();
        
        for rel in &self.schema.relationships {
            // Check if tables exist
            let from_table = match table_map.get(&rel.from_table) {
                Some(t) => t,
                None => {
                    errors.push(ValidationError::TableNotFound(rel.from_table.clone()));
                    continue;
                }
            };
            
            let to_table = match table_map.get(&rel.to_table) {
                Some(t) => t,
                None => {
                    errors.push(ValidationError::TableNotFound(rel.to_table.clone()));
                    continue;
                }
            };
            
            // Check if columns exist
            if !from_table.columns.iter().any(|c| c.name == rel.from_field) {
                errors.push(ValidationError::ColumnNotFound {
                    table: rel.from_table.clone(),
                    column: rel.from_field.clone(),
                });
            }
            
            if !to_table.columns.iter().any(|c| c.name == rel.to_field) {
                errors.push(ValidationError::ColumnNotFound {
                    table: rel.to_table.clone(),
                    column: rel.to_field.clone(),
                });
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    pub fn print_schema(&self) {
        if let Some(title) = &self.schema.title {
            println!("📊 ERD: {}", title);
            println!("{}", "=".repeat(60));
        }
        
        println!("\n📋 Tables:");
        for table in &self.schema.tables {
            println!("\n  ┌─ Table: {}", table.name);
            for column in &table.columns {
                let attrs = if column.attributes.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", 
                        column.attributes.iter()
                            .map(|a| format!("{}", a))
                            .collect::<Vec<_>>()
                            .join(", "))
                };
                println!("  │  • {}: {}{}", column.name, column.datatype, attrs);
            }
            println!("  └─");
        }
        
        if !self.schema.relationships.is_empty() {
            println!("\n🔗 Relationships:");
            for rel in &self.schema.relationships {
                let arrow = match rel.relationship_type {
                    RelationshipType::OneToMany => "──>",
                    RelationshipType::ManyToOne => "<──",
                    RelationshipType::ManyToMany => "<─>",
                    RelationshipType::OneToOne => "───",
                };
                println!("  {}.{} {} {}.{} ({})",
                    rel.from_table, rel.from_field,
                    arrow,
                    rel.to_table, rel.to_field,
                    rel.relationship_type);
            }
        }
        
        println!();
    }
    
    pub fn get_statistics(&self) -> SchemaStatistics {
        let total_columns: usize = self.schema.tables.iter()
            .map(|t| t.columns.len())
            .sum();
        
        let primary_keys: usize = self.schema.tables.iter()
            .flat_map(|t| &t.columns)
            .filter(|c| c.is_primary_key())
            .count();
        
        let foreign_keys: usize = self.schema.tables.iter()
            .flat_map(|t| &t.columns)
            .filter(|c| c.is_foreign_key())
            .count();
        
        SchemaStatistics {
            table_count: self.schema.tables.len(),
            total_columns,
            relationship_count: self.schema.relationships.len(),
            primary_keys,
            foreign_keys,
        }
    }
}

#[derive(Debug)]
pub struct SchemaStatistics {
    pub table_count: usize,
    pub total_columns: usize,
    pub relationship_count: usize,
    pub primary_keys: usize,
    pub foreign_keys: usize,
}

impl SchemaStatistics {
    pub fn print(&self) {
        println!("📊 Schema Statistics:");
        println!("  Tables: {}", self.table_count);
        println!("  Columns: {}", self.total_columns);
        println!("  Relationships: {}", self.relationship_count);
        println!("  Primary Keys: {}", self.primary_keys);
        println!("  Foreign Keys: {}", self.foreign_keys);
    }
}
