use std::fmt;
use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub title: Option<String>,
    pub tables: Vec<Table>,
    pub relationships: Vec<Relationship>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Schema {
    pub fn new() -> Self {
        Schema {
            title: None,
            tables: Vec::new(),
            relationships: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub span: Option<Span>,
}

impl Table {
    pub fn with_span(name: String, span: Span) -> Self {
        Table {
            name,
            columns: Vec::new(),
            span: Some(span),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub datatype: DataType,
    pub attributes: Vec<Attribute>,
    pub span: Option<Span>,
}

impl Column {
    pub fn with_span(name: String, datatype: DataType, span: Span) -> Self {
        Column {
            name,
            datatype,
            attributes: Vec::new(),
            span: Some(span),
        }
    }
    
    pub fn is_primary_key(&self) -> bool {
        self.attributes.iter().any(|a| matches!(a, Attribute::PrimaryKey))
    }
    
    pub fn is_foreign_key(&self) -> bool {
        self.attributes.iter().any(|a| matches!(a, Attribute::ForeignKey))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    String,
    Int,
    Bool,
    Double,
    Float,
    Decimal,
    Date,
    Time,
    DateTime,
    Blob,
    TinyBlob,
    LargeBlob,
    Custom(String),
}

impl DataType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "string" | "str" => DataType::String,
            "int" | "integer" => DataType::Int,
            "bool" | "boolean" => DataType::Bool,
            "double" => DataType::Double,
            "float" => DataType::Float,
            "decimal" => DataType::Decimal,
            "date" => DataType::Date,
            "time" => DataType::Time,
            "datetime" => DataType::DateTime,
            "blob" => DataType::Blob,
            "tinyblob" => DataType::TinyBlob,
            "largeblob" => DataType::LargeBlob,
            _ => DataType::Custom(s.to_string()),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::String => write!(f, "str"),
            DataType::Int => write!(f, "int"),
            DataType::Bool => write!(f, "bool"),
            DataType::Double => write!(f, "double"),
            DataType::Float => write!(f, "float"),
            DataType::Decimal => write!(f, "decimal"),
            DataType::Date => write!(f, "date"),
            DataType::Time => write!(f, "time"),
            DataType::DateTime => write!(f, "datetime"),
            DataType::Blob => write!(f, "blob"),
            DataType::TinyBlob => write!(f, "tinyblob"),
            DataType::LargeBlob => write!(f, "largeblob"),
            DataType::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
    PrimaryKey,
    ForeignKey,
    Unique,
    Nullable,
    Indexed,
    Default(DefaultValue),
    AutoIncrement,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Attribute::PrimaryKey => write!(f, "pk"),
            Attribute::ForeignKey => write!(f, "fk"),
            Attribute::Unique => write!(f, "unique"),
            Attribute::Nullable => write!(f, "nullable"),
            Attribute::Indexed => write!(f, "indexed"),
            Attribute::Default(v) => write!(f, "default={}", v),
            Attribute::AutoIncrement => write!(f, "autoincrement"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    Now,
    True,
    False,
    Null,
    String(String),
    Number(i64),
}

impl fmt::Display for DefaultValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DefaultValue::Now => write!(f, "NOW"),
            DefaultValue::True => write!(f, "TRUE"),
            DefaultValue::False => write!(f, "FALSE"),
            DefaultValue::Null => write!(f, "NULL"),
            DefaultValue::String(s) => write!(f, "\"{}\"", s),
            DefaultValue::Number(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    pub from_table: String,
    pub from_field: String,
    pub to_table: String,
    pub to_field: String,
    pub relationship_type: RelationshipType,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelationshipType {
    OneToMany,      // >
    ManyToOne,      // <
    ManyToMany,     // <>
    OneToOne,       // -
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RelationshipType::OneToMany => write!(f, "one-to-many"),
            RelationshipType::ManyToOne => write!(f, "many-to-one"),
            RelationshipType::ManyToMany => write!(f, "many-to-many"),
            RelationshipType::OneToOne => write!(f, "one-to-one"),
        }
    }
}

// Node structure for graph databases
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub fields: Vec<NodeField>,
    pub span: Option<Span>,
}

impl Node {
    pub fn with_span(name: String, span: Span) -> Self {
        Node {
            name,
            fields: Vec::new(),
            span: Some(span),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeField {
    pub name: String,
    pub datatype: DataType,
    pub attributes: Vec<Attribute>,
    pub span: Option<Span>,
}

impl NodeField {
    pub fn with_span(name: String, datatype: DataType, span: Span) -> Self {
        NodeField {
            name,
            datatype,
            attributes: Vec::new(),
            span: Some(span),
        }
    }
}

// Edge structure for graph databases
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub name: String,
    pub from_node: String,
    pub to_node: String,
    pub edge_type: EdgeType,
    pub properties: Vec<EdgeProperty>,
    pub attributes: Vec<Attribute>,
    pub span: Option<Span>,
}

impl Edge {
    pub fn new(name: String, from_node: String, to_node: String, edge_type: EdgeType) -> Self {
        Edge {
            name,
            from_node,
            to_node,
            edge_type,
            properties: Vec::new(),
            attributes: Vec::new(),
            span: None,
        }
    }

    pub fn with_span(name: String, from_node: String, to_node: String, edge_type: EdgeType, span: Span) -> Self {
        Edge {
            name,
            from_node,
            to_node,
            edge_type,
            properties: Vec::new(),
            attributes: Vec::new(),
            span: Some(span),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeProperty {
    pub name: String,
    pub datatype: DataType,
    pub attributes: Vec<Attribute>,
    pub span: Option<Span>,
}

impl EdgeProperty {
    pub fn with_span(name: String, datatype: DataType, span: Span) -> Self {
        EdgeProperty {
            name,
            datatype,
            attributes: Vec::new(),
            span: Some(span),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeType {
    Outgoing,       // -[]->
    Incoming,       // <-[]-
    Bidirectional,  // <-[]->
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EdgeType::Outgoing => write!(f, "outgoing"),
            EdgeType::Incoming => write!(f, "incoming"),
            EdgeType::Bidirectional => write!(f, "bidirectional"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_new() {
        let schema = Schema::new();
        assert!(schema.title.is_none());
        assert_eq!(schema.tables.len(), 0);
        assert_eq!(schema.relationships.len(), 0);
    }

    #[test]
    fn test_datatype_display() {
        assert_eq!(DataType::String.to_string(), "str");
        assert_eq!(DataType::Int.to_string(), "int");
        assert_eq!(DataType::Bool.to_string(), "bool");
        assert_eq!(DataType::DateTime.to_string(), "datetime");
        assert_eq!(DataType::Custom("uuid".to_string()).to_string(), "uuid");
    }

    #[test]
    fn test_attribute_display() {
        assert_eq!(Attribute::PrimaryKey.to_string(), "pk");
        assert_eq!(Attribute::ForeignKey.to_string(), "fk");
        assert_eq!(Attribute::Unique.to_string(), "unique");
        assert_eq!(Attribute::Nullable.to_string(), "nullable");
        assert_eq!(Attribute::AutoIncrement.to_string(), "autoincrement");
    }

    #[test]
    fn test_default_value_display() {
        assert_eq!(DefaultValue::Now.to_string(), "NOW");
        assert_eq!(DefaultValue::True.to_string(), "TRUE");
        assert_eq!(DefaultValue::False.to_string(), "FALSE");
        assert_eq!(DefaultValue::Null.to_string(), "NULL");
        assert_eq!(DefaultValue::String("test".to_string()).to_string(), "\"test\"");
        assert_eq!(DefaultValue::Number(42).to_string(), "42");
    }

    #[test]
    fn test_relationship_type_display() {
        assert_eq!(RelationshipType::OneToMany.to_string(), "one-to-many");
        assert_eq!(RelationshipType::ManyToOne.to_string(), "many-to-one");
        assert_eq!(RelationshipType::ManyToMany.to_string(), "many-to-many");
        assert_eq!(RelationshipType::OneToOne.to_string(), "one-to-one");
    }

    #[test]
    fn test_table_creation() {
        let span = Span { line: 1, column: 1, length: 10 };
        let table = Table::with_span("Users".to_string(), span);
        assert_eq!(table.name, "Users");
        assert_eq!(table.columns.len(), 0);
        assert!(table.span.is_some());
    }

    #[test]
    fn test_column_creation() {
        let span = Span { line: 1, column: 1, length: 10 };
        let column = Column::with_span("id".to_string(), DataType::Int, span);
        assert_eq!(column.name, "id");
        assert_eq!(column.datatype, DataType::Int);
        assert_eq!(column.attributes.len(), 0);
        assert!(column.span.is_some());
    }

    #[test]
    fn test_schema_clone() {
        let mut schema = Schema::new();
        schema.title = Some("Test".to_string());
        
        let cloned = schema.clone();
        assert_eq!(schema.title, cloned.title);
    }

    #[test]
    fn test_datatype_equality() {
        assert_eq!(DataType::Int, DataType::Int);
        assert_ne!(DataType::Int, DataType::String);
        assert_eq!(DataType::Custom("uuid".to_string()), DataType::Custom("uuid".to_string()));
    }

    #[test]
    fn test_attribute_equality() {
        assert_eq!(Attribute::PrimaryKey, Attribute::PrimaryKey);
        assert_ne!(Attribute::PrimaryKey, Attribute::ForeignKey);
        
        let default1 = Attribute::Default(DefaultValue::Now);
        let default2 = Attribute::Default(DefaultValue::Now);
        assert_eq!(default1, default2);
    }

    #[test]
    fn test_relationship_type_equality() {
        assert_eq!(RelationshipType::OneToMany, RelationshipType::OneToMany);
        assert_ne!(RelationshipType::OneToMany, RelationshipType::ManyToOne);
    }
    
    #[test]
    fn test_decimal_datatype() {
        assert_eq!(DataType::Decimal.to_string(), "decimal");
        assert_eq!(DataType::from_str("decimal"), DataType::Decimal);
    }
    
    #[test]
    fn test_indexed_attribute() {
        assert_eq!(Attribute::Indexed.to_string(), "indexed");
    }
    
    #[test]
    fn test_node_creation() {
        let span = Span { line: 1, column: 1, length: 10 };
        let node = Node::with_span("Person".to_string(), span);
        assert_eq!(node.name, "Person");
        assert_eq!(node.fields.len(), 0);
        assert!(node.span.is_some());
    }
    
    #[test]
    fn test_edge_creation() {
        let span = Span { line: 1, column: 1, length: 10 };
        let edge = Edge::with_span(
            "WORKS_AT".to_string(),
            "Person".to_string(),
            "Company".to_string(),
            EdgeType::Outgoing,
            span
        );
        assert_eq!(edge.name, "WORKS_AT");
        assert_eq!(edge.from_node, "Person");
        assert_eq!(edge.to_node, "Company");
        assert_eq!(edge.edge_type, EdgeType::Outgoing);
    }
    
    #[test]
    fn test_edge_type_display() {
        assert_eq!(EdgeType::Outgoing.to_string(), "outgoing");
        assert_eq!(EdgeType::Incoming.to_string(), "incoming");
        assert_eq!(EdgeType::Bidirectional.to_string(), "bidirectional");
    }
}
