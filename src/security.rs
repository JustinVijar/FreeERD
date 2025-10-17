// Security limits and validation for FreeERD schemas

use crate::ast::Schema;
use std::collections::{HashSet, HashMap};

pub const MAX_TABLES: usize = 5000;
pub const MAX_COLUMNS_PER_TABLE: usize = 1000;
pub const MAX_RELATIONSHIPS: usize = 50000;
pub const MAX_STRING_LENGTH: usize = 5000;
pub const MAX_IDENTIFIER_LENGTH: usize = 500;
pub const MAX_CYCLE_DEPTH: usize = 500;

#[derive(Debug)]
pub enum SecurityError {
    TooManyTables(usize),
    TooManyColumns(String, usize),
    TooManyRelationships(usize),
    StringTooLong(String),
    IdentifierTooLong(String),
    CyclicRelationship(String),
    InvalidIdentifier(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SecurityError::TooManyTables(count) => {
                write!(f, "Too many tables: {} (max {})", count, MAX_TABLES)
            }
            SecurityError::TooManyColumns(table, count) => {
                write!(f, "Too many columns in table '{}': {} (max {})", table, count, MAX_COLUMNS_PER_TABLE)
            }
            SecurityError::TooManyRelationships(count) => {
                write!(f, "Too many relationships: {} (max {})", count, MAX_RELATIONSHIPS)
            }
            SecurityError::StringTooLong(s) => {
                write!(f, "String too long: '{}...' (max {} chars)", &s[..50.min(s.len())], MAX_STRING_LENGTH)
            }
            SecurityError::IdentifierTooLong(id) => {
                write!(f, "Identifier too long: '{}' (max {} chars)", id, MAX_IDENTIFIER_LENGTH)
            }
            SecurityError::CyclicRelationship(path) => {
                write!(f, "Cyclic relationship detected: {}", path)
            }
            SecurityError::InvalidIdentifier(id) => {
                write!(f, "Invalid identifier '{}': must be alphanumeric or underscore, starting with letter or underscore", id)
            }
        }
    }
}

impl std::error::Error for SecurityError {}

pub struct SecurityValidator;

impl SecurityValidator {
    /// Validate schema against security constraints
    pub fn validate(schema: &Schema) -> Result<(), SecurityError> {
        // Check table count
        if schema.tables.len() > MAX_TABLES {
            return Err(SecurityError::TooManyTables(schema.tables.len()));
        }

        // Check relationship count
        if schema.relationships.len() > MAX_RELATIONSHIPS {
            return Err(SecurityError::TooManyRelationships(schema.relationships.len()));
        }

        // Validate each table
        for table in &schema.tables {
            // Check column count
            if table.columns.len() > MAX_COLUMNS_PER_TABLE {
                return Err(SecurityError::TooManyColumns(
                    table.name.clone(),
                    table.columns.len()
                ));
            }

            // Validate table name
            Self::validate_identifier(&table.name)?;

            // Validate column names
            for column in &table.columns {
                Self::validate_identifier(&column.name)?;
            }
        }

        // Validate relationship identifiers
        for rel in &schema.relationships {
            Self::validate_identifier(&rel.from_table)?;
            Self::validate_identifier(&rel.to_table)?;
            Self::validate_identifier(&rel.from_field)?;
            Self::validate_identifier(&rel.to_field)?;
        }

        // Check for cycles in relationships
        Self::detect_cycles(schema)?;

        // Validate title if present
        if let Some(title) = &schema.title {
            Self::validate_string(title)?;
        }

        Ok(())
    }

    /// Validate identifier (table/column names)
    fn validate_identifier(id: &str) -> Result<(), SecurityError> {
        // Check length
        if id.len() > MAX_IDENTIFIER_LENGTH {
            return Err(SecurityError::IdentifierTooLong(id.to_string()));
        }

        // Check for valid characters (alphanumeric + underscore)
        if !id.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(SecurityError::InvalidIdentifier(id.to_string()));
        }

        // Must start with letter or underscore
        if let Some(first) = id.chars().next() {
            if !first.is_alphabetic() && first != '_' {
                return Err(SecurityError::InvalidIdentifier(id.to_string()));
            }
        }

        Ok(())
    }

    /// Validate string length
    fn validate_string(s: &str) -> Result<(), SecurityError> {
        if s.len() > MAX_STRING_LENGTH {
            return Err(SecurityError::StringTooLong(s.to_string()));
        }
        Ok(())
    }

    /// Detect cycles in relationships using DFS
    fn detect_cycles(schema: &Schema) -> Result<(), SecurityError> {
        // Build adjacency list
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        
        for rel in &schema.relationships {
            graph.entry(rel.from_table.clone())
                .or_insert_with(Vec::new)
                .push(rel.to_table.clone());
        }

        // Track visited nodes and recursion stack
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();
        let mut path: Vec<String> = Vec::new();

        // Check each table as potential starting point
        for table in &schema.tables {
            if !visited.contains(&table.name) {
                if Self::dfs_cycle_detect(
                    &table.name,
                    &graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    0
                )? {
                    return Err(SecurityError::CyclicRelationship(
                        path.join(" -> ")
                    ));
                }
            }
        }

        Ok(())
    }

    /// DFS helper for cycle detection
    fn dfs_cycle_detect(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        depth: usize,
    ) -> Result<bool, SecurityError> {
        // Prevent stack overflow from deep recursion
        if depth > MAX_CYCLE_DEPTH {
            return Err(SecurityError::CyclicRelationship(
                format!("Maximum depth {} exceeded", MAX_CYCLE_DEPTH)
            ));
        }

        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        // Check all neighbors
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                // Allow self-referencing (e.g., Categories.id > Categories.parent_id)
                if neighbor == node {
                    continue;  // Skip self-loops, they're valid for hierarchical data
                }
                
                if !visited.contains(neighbor) {
                    if Self::dfs_cycle_detect(
                        neighbor,
                        graph,
                        visited,
                        rec_stack,
                        path,
                        depth + 1
                    )? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Cycle detected (not a self-loop)
                    path.push(neighbor.to_string());
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Table, Column, Relationship, RelationshipType, DataType};

    #[test]
    fn test_valid_schema() {
        let schema = Schema {
            title: Some("Test".to_string()),
            tables: vec![
                Table {
                    name: "Users".to_string(),
                    columns: vec![
                        Column {
                            name: "id".to_string(),
                            datatype: DataType::Int,
                            attributes: vec![],
                            span: None,
                        }
                    ],
                    span: None,
                }
            ],
            relationships: vec![],
        };

        assert!(SecurityValidator::validate(&schema).is_ok());
    }

    #[test]
    fn test_too_many_tables() {
        let mut tables = Vec::new();
        for i in 0..5001 {  // Updated to exceed new limit of 5000
            tables.push(Table {
                name: format!("Table{}", i),
                columns: vec![],
                span: None,
            });
        }

        let schema = Schema {
            title: None,
            tables,
            relationships: vec![],
        };

        assert!(matches!(
            SecurityValidator::validate(&schema),
            Err(SecurityError::TooManyTables(_))
        ));
    }

    #[test]
    fn test_invalid_identifier() {
        let schema = Schema {
            title: None,
            tables: vec![
                Table {
                    name: "Invalid-Name!".to_string(),
                    columns: vec![],
                    span: None,
                }
            ],
            relationships: vec![],
        };

        assert!(matches!(
            SecurityValidator::validate(&schema),
            Err(SecurityError::InvalidIdentifier(_))
        ));
    }

    #[test]
    fn test_cycle_detection() {
        let schema = Schema {
            title: None,
            tables: vec![
                Table { name: "A".to_string(), columns: vec![], span: None },
                Table { name: "B".to_string(), columns: vec![], span: None },
            ],
            relationships: vec![
                Relationship {
                    from_table: "A".to_string(),
                    from_field: "id".to_string(),
                    to_table: "B".to_string(),
                    to_field: "id".to_string(),
                    relationship_type: RelationshipType::OneToMany,
                    span: None,
                },
                Relationship {
                    from_table: "B".to_string(),
                    from_field: "id".to_string(),
                    to_table: "A".to_string(),
                    to_field: "id".to_string(),
                    relationship_type: RelationshipType::OneToMany,
                    span: None,
                },
            ],
        };

        assert!(matches!(
            SecurityValidator::validate(&schema),
            Err(SecurityError::CyclicRelationship(_))
        ));
    }
}
