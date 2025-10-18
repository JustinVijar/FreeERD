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
    DuplicateNode { name: String, span: Option<Span> },
    DuplicateNodeField { node: String, field: String, span: Option<Span> },
    NodeNotFound { name: String, span: Option<Span> },
    DuplicateEdge { name: String, span: Option<Span> },
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
            ValidationError::DuplicateNode { name, .. } => {
                write!(f, "Duplicate node definition: {}", name)
            }
            ValidationError::DuplicateNodeField { node, field, .. } => {
                write!(f, "Duplicate field '{}' in node '{}'", field, node)
            }
            ValidationError::NodeNotFound { name, .. } => {
                write!(f, "Node '{}' not found", name)
            }
            ValidationError::DuplicateEdge { name, .. } => {
                write!(f, "Duplicate edge definition: {}", name)
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
            ValidationError::DuplicateNode { span, .. } => *span,
            ValidationError::DuplicateNodeField { span, .. } => *span,
            ValidationError::NodeNotFound { span, .. } => *span,
            ValidationError::DuplicateEdge { span, .. } => *span,
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
        
        // Validate nodes
        if let Err(e) = self.validate_nodes() {
            errors.extend(e);
        }
        
        // Validate edges
        if let Err(e) = self.validate_edges() {
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
    
    fn validate_nodes(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut node_names = HashSet::new();
        
        for node in &self.schema.nodes {
            // Check for duplicate nodes
            if !node_names.insert(&node.name) {
                errors.push(ValidationError::DuplicateNode {
                    name: node.name.clone(),
                    span: node.span,
                });
                continue;
            }
            
            // Check for duplicate fields
            let mut field_names = HashSet::new();
            for field in &node.fields {
                if !field_names.insert(&field.name) {
                    errors.push(ValidationError::DuplicateNodeField {
                        node: node.name.clone(),
                        field: field.name.clone(),
                        span: field.span,
                    });
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    fn validate_edges(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut edge_names = HashSet::new();
        let node_map: HashMap<_, _> = self.schema.nodes.iter()
            .map(|n| (&n.name, n))
            .collect();
        
        for edge in &self.schema.edges {
            // Check for duplicate edges
            if !edge_names.insert(&edge.name) {
                errors.push(ValidationError::DuplicateEdge {
                    name: edge.name.clone(),
                    span: edge.span,
                });
                continue;
            }
            
            // Check if nodes exist
            if !node_map.contains_key(&edge.from_node) {
                errors.push(ValidationError::NodeNotFound {
                    name: edge.from_node.clone(),
                    span: edge.span,
                });
            }
            
            if !node_map.contains_key(&edge.to_node) {
                errors.push(ValidationError::NodeNotFound {
                    name: edge.to_node.clone(),
                    span: edge.span,
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
        
        let total_node_fields: usize = self.schema.nodes.iter()
            .map(|n| n.fields.len())
            .sum();
        
        let total_edge_properties: usize = self.schema.edges.iter()
            .map(|e| e.properties.len())
            .sum();
        
        SchemaStatistics {
            table_count: self.schema.tables.len(),
            total_columns,
            relationship_count: self.schema.relationships.len(),
            primary_keys,
            foreign_keys,
            node_count: self.schema.nodes.len(),
            total_node_fields,
            edge_count: self.schema.edges.len(),
            total_edge_properties,
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
    pub node_count: usize,
    pub total_node_fields: usize,
    pub edge_count: usize,
    pub total_edge_properties: usize,
}

impl SchemaStatistics {
    pub fn print(&self) {
        println!("📊 Schema Statistics:");
        println!("  Tables: {}", self.table_count);
        println!("  Columns: {}", self.total_columns);
        println!("  Relationships: {}", self.relationship_count);
        println!("  Primary Keys: {}", self.primary_keys);
        println!("  Foreign Keys: {}", self.foreign_keys);
        println!("  Nodes: {}", self.node_count);
        println!("  Node Fields: {}", self.total_node_fields);
        println!("  Edges: {}", self.edge_count);
        println!("  Edge Properties: {}", self.total_edge_properties);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_table(name: &str, columns: Vec<(&str, DataType, Vec<Attribute>)>) -> Table {
        Table {
            name: name.to_string(),
            columns: columns.into_iter().map(|(col_name, dtype, attrs)| Column {
                name: col_name.to_string(),
                datatype: dtype,
                attributes: attrs,
                span: None,
            }).collect(),
            span: None,
        }
    }

    #[test]
    fn test_valid_schema() {
        let schema = Schema {
            title: Some("Test".to_string()),
            tables: vec![
                create_test_table("Users", vec![
                    ("id", DataType::Int, vec![Attribute::PrimaryKey]),
                    ("name", DataType::String, vec![]),
                ]),
            ],
            relationships: vec![],
            nodes: vec![],
            edges: vec![],
        };

        let interpreter = Interpreter::new(schema);
        assert!(interpreter.validate().is_ok());
    }

    #[test]
    fn test_duplicate_table() {
        let schema = Schema {
            title: None,
            tables: vec![
                create_test_table("Users", vec![]),
                create_test_table("Users", vec![]),
            ],
            relationships: vec![],
            nodes: vec![],
            edges: vec![],
        };

        let interpreter = Interpreter::new(schema);
        let result = interpreter.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::DuplicateTable { .. }));
    }

    #[test]
    fn test_duplicate_column() {
        let schema = Schema {
            title: None,
            tables: vec![
                create_test_table("Users", vec![
                    ("id", DataType::Int, vec![]),
                    ("id", DataType::String, vec![]),
                ]),
            ],
            relationships: vec![],
            nodes: vec![],
            edges: vec![],
        };

        let interpreter = Interpreter::new(schema);
        let result = interpreter.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::DuplicateColumn { .. }));
    }

    #[test]
    fn test_table_not_found_in_relationship() {
        let schema = Schema {
            title: None,
            tables: vec![
                create_test_table("Users", vec![
                    ("id", DataType::Int, vec![Attribute::PrimaryKey]),
                ]),
            ],
            relationships: vec![
                Relationship {
                    from_table: "Users".to_string(),
                    from_field: "id".to_string(),
                    to_table: "Posts".to_string(),
                    to_field: "user_id".to_string(),
                    relationship_type: RelationshipType::OneToMany,
                    span: None,
                },
            ],
            nodes: vec![],
            edges: vec![],
        };

        let interpreter = Interpreter::new(schema);
        let result = interpreter.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ValidationError::TableNotFound { .. })));
    }

    #[test]
    fn test_column_not_found_in_relationship() {
        let schema = Schema {
            title: None,
            tables: vec![
                create_test_table("Users", vec![
                    ("id", DataType::Int, vec![Attribute::PrimaryKey]),
                ]),
                create_test_table("Posts", vec![
                    ("id", DataType::Int, vec![Attribute::PrimaryKey]),
                ]),
            ],
            relationships: vec![
                Relationship {
                    from_table: "Users".to_string(),
                    from_field: "nonexistent".to_string(),
                    to_table: "Posts".to_string(),
                    to_field: "id".to_string(),
                    relationship_type: RelationshipType::OneToMany,
                    span: None,
                },
            ],
            nodes: vec![],
            edges: vec![],
        };

        let interpreter = Interpreter::new(schema);
        let result = interpreter.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ValidationError::ColumnNotFound { .. })));
    }

    #[test]
    fn test_statistics() {
        let schema = Schema {
            title: None,
            tables: vec![
                create_test_table("Users", vec![
                    ("id", DataType::Int, vec![Attribute::PrimaryKey]),
                    ("email", DataType::String, vec![Attribute::Unique]),
                ]),
                create_test_table("Posts", vec![
                    ("id", DataType::Int, vec![Attribute::PrimaryKey]),
                    ("user_id", DataType::Int, vec![Attribute::ForeignKey]),
                ]),
            ],
            relationships: vec![
                Relationship {
                    from_table: "Users".to_string(),
                    from_field: "id".to_string(),
                    to_table: "Posts".to_string(),
                    to_field: "user_id".to_string(),
                    relationship_type: RelationshipType::OneToMany,
                    span: None,
                },
            ],
            nodes: vec![],
            edges: vec![],
        };

        let interpreter = Interpreter::new(schema);
        let stats = interpreter.get_statistics();
        
        assert_eq!(stats.table_count, 2);
        assert_eq!(stats.total_columns, 4);
        assert_eq!(stats.relationship_count, 1);
        assert_eq!(stats.primary_keys, 2);
        assert_eq!(stats.foreign_keys, 1);
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::DuplicateTable {
            name: "Users".to_string(),
            span: None,
        };
        assert_eq!(error.to_string(), "Duplicate table definition: Users");

        let error = ValidationError::TableNotFound {
            name: "Posts".to_string(),
            span: None,
        };
        assert_eq!(error.to_string(), "Table 'Posts' not found");
    }

    #[test]
    fn test_multiple_validation_errors() {
        let schema = Schema {
            title: None,
            tables: vec![
                create_test_table("Users", vec![
                    ("id", DataType::Int, vec![]),
                    ("id", DataType::String, vec![]), // Duplicate column
                ]),
                create_test_table("Users", vec![]), // Duplicate table
            ],
            relationships: vec![],
            nodes: vec![],
            edges: vec![],
        };

        let interpreter = Interpreter::new(schema);
        let result = interpreter.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2); // At least duplicate table and column errors
    }
    
    #[test]
    fn test_valid_nodes_and_edges() {
        let schema = Schema {
            title: None,
            tables: vec![],
            relationships: vec![],
            nodes: vec![
                Node {
                    name: "Person".to_string(),
                    fields: vec![
                        NodeField {
                            name: "id".to_string(),
                            datatype: DataType::Int,
                            attributes: vec![Attribute::PrimaryKey],
                            span: None,
                        },
                    ],
                    span: None,
                },
                Node {
                    name: "Company".to_string(),
                    fields: vec![],
                    span: None,
                },
            ],
            edges: vec![
                Edge {
                    name: "WORKS_AT".to_string(),
                    from_node: "Person".to_string(),
                    to_node: "Company".to_string(),
                    edge_type: EdgeType::Outgoing,
                    properties: vec![],
                    attributes: vec![],
                    span: None,
                },
            ],
        };

        let interpreter = Interpreter::new(schema);
        assert!(interpreter.validate().is_ok());
    }
    
    #[test]
    fn test_duplicate_node() {
        let schema = Schema {
            title: None,
            tables: vec![],
            relationships: vec![],
            nodes: vec![
                Node {
                    name: "Person".to_string(),
                    fields: vec![],
                    span: None,
                },
                Node {
                    name: "Person".to_string(),
                    fields: vec![],
                    span: None,
                },
            ],
            edges: vec![],
        };

        let interpreter = Interpreter::new(schema);
        let result = interpreter.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::DuplicateNode { .. }));
    }
    
    #[test]
    fn test_node_not_found_in_edge() {
        let schema = Schema {
            title: None,
            tables: vec![],
            relationships: vec![],
            nodes: vec![],
            edges: vec![
                Edge {
                    name: "WORKS_AT".to_string(),
                    from_node: "Person".to_string(),
                    to_node: "Company".to_string(),
                    edge_type: EdgeType::Outgoing,
                    properties: vec![],
                    attributes: vec![],
                    span: None,
                },
            ],
        };

        let interpreter = Interpreter::new(schema);
        let result = interpreter.validate();
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2); // Both nodes not found
        assert!(errors.iter().any(|e| matches!(e, ValidationError::NodeNotFound { .. })));
    }
    
    #[test]
    fn test_statistics_with_nodes_and_edges() {
        let schema = Schema {
            title: None,
            tables: vec![],
            relationships: vec![],
            nodes: vec![
                Node {
                    name: "Person".to_string(),
                    fields: vec![
                        NodeField {
                            name: "id".to_string(),
                            datatype: DataType::Int,
                            attributes: vec![],
                            span: None,
                        },
                        NodeField {
                            name: "name".to_string(),
                            datatype: DataType::String,
                            attributes: vec![],
                            span: None,
                        },
                    ],
                    span: None,
                },
            ],
            edges: vec![
                Edge {
                    name: "KNOWS".to_string(),
                    from_node: "Person".to_string(),
                    to_node: "Person".to_string(),
                    edge_type: EdgeType::Bidirectional,
                    properties: vec![
                        EdgeProperty {
                            name: "since".to_string(),
                            datatype: DataType::Date,
                            attributes: vec![],
                            span: None,
                        },
                    ],
                    attributes: vec![],
                    span: None,
                },
            ],
        };

        let interpreter = Interpreter::new(schema);
        let stats = interpreter.get_statistics();
        
        assert_eq!(stats.node_count, 1);
        assert_eq!(stats.total_node_fields, 2);
        assert_eq!(stats.edge_count, 1);
        assert_eq!(stats.total_edge_properties, 1);
    }
}
