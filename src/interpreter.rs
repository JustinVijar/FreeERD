use crate::ast::*;
use crate::lexer::Span;
use std::collections::{HashMap, HashSet};

pub struct Interpreter {
    schema: Schema,
}

#[derive(Debug)]
pub enum ValidationError {
    DuplicateTable { name: String, span: Option<Span> },
    DuplicateColumn { table: String, column: String, span: Option<Span> },
    TableNotFound { name: String, span: Option<Span> },
    ColumnNotFound { table: String, column: String, span: Option<Span> },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateTable { name, .. } => {
                write!(f, "Duplicate table definition: {}", name)
            }
            ValidationError::DuplicateColumn { table, column, .. } => {
                write!(f, "Duplicate column '{}' in table '{}'", column, table)
            }
            ValidationError::TableNotFound { name, .. } => {
                write!(f, "Table '{}' not found", name)
            }
            ValidationError::ColumnNotFound { table, column, .. } => {
                write!(f, "Column '{}' not found in table '{}'", column, table)
            }
        }
    }
}

impl ValidationError {
    pub fn span(&self) -> Option<Span> {
        match self {
            ValidationError::DuplicateTable { span, .. } => *span,
            ValidationError::DuplicateColumn { span, .. } => *span,
            ValidationError::TableNotFound { span, .. } => *span,
            ValidationError::ColumnNotFound { span, .. } => *span,
        }
    }
    
    /// Format error with Rust-like error messages including line numbers and source context
    pub fn format_with_source(&self, source: &str, filename: &str) -> String {
        let mut output = String::new();
        
        // Error header
        output.push_str(&format!("\x1b[1;31merror\x1b[0m: {}\n", self));
        
        if let Some(span) = self.span() {
            // Location info
            output.push_str(&format!("  \x1b[1;34m-->\x1b[0m {}:{}:{}\n", filename, span.line, span.column));
            output.push_str("   \x1b[1;34m|\x1b[0m\n");
            
            // Get the source line
            if let Some(line_text) = get_source_line(source, span.line) {
                // Line number and source
                output.push_str(&format!(" \x1b[1;34m{:>3} |\x1b[0m {}\n", span.line, line_text));
                
                // Error indicator (^^^)
                output.push_str("   \x1b[1;34m|\x1b[0m ");
                output.push_str(&" ".repeat(span.column - 1));
                output.push_str(&format!("\x1b[1;31m{}\x1b[0m", "^".repeat(span.length.max(1))));
                output.push_str("\n");
            }
        }
        
        output
    }
}

fn get_source_line(source: &str, line_number: usize) -> Option<String> {
    source.lines().nth(line_number - 1).map(|s| s.to_string())
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
                errors.push(ValidationError::DuplicateTable {
                    name: table.name.clone(),
                    span: table.span,
                });
                continue;
            }
            
            // Check for duplicate columns
            let mut column_names = HashSet::new();
            for column in &table.columns {
                if !column_names.insert(&column.name) {
                    errors.push(ValidationError::DuplicateColumn {
                        table: table.name.clone(),
                        column: column.name.clone(),
                        span: column.span,
                    });
                }
            }
            
            // Note: Multiple primary keys are allowed (composite primary keys)
            // where multiple columns together form the primary key
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
                    errors.push(ValidationError::TableNotFound {
                        name: rel.from_table.clone(),
                        span: rel.span,
                    });
                    continue;
                }
            };
            
            let to_table = match table_map.get(&rel.to_table) {
                Some(t) => t,
                None => {
                    errors.push(ValidationError::TableNotFound {
                        name: rel.to_table.clone(),
                        span: rel.span,
                    });
                    continue;
                }
            };
            
            // Check if columns exist
            if !from_table.columns.iter().any(|c| c.name == rel.from_field) {
                errors.push(ValidationError::ColumnNotFound {
                    table: rel.from_table.clone(),
                    column: rel.from_field.clone(),
                    span: rel.span,
                });
            }
            
            if !to_table.columns.iter().any(|c| c.name == rel.to_field) {
                errors.push(ValidationError::ColumnNotFound {
                    table: rel.to_table.clone(),
                    column: rel.to_field.clone(),
                    span: rel.span,
                });
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
