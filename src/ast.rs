use std::fmt;
use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub title: Option<String>,
    pub tables: Vec<Table>,
    pub relationships: Vec<Relationship>,
}

impl Schema {
    pub fn new() -> Self {
        Schema {
            title: None,
            tables: Vec::new(),
            relationships: Vec::new(),
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
    pub fn new(name: String) -> Self {
        Table {
            name,
            columns: Vec::new(),
            span: None,
        }
    }
    
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
    pub fn new(name: String, datatype: DataType) -> Self {
        Column {
            name,
            datatype,
            attributes: Vec::new(),
            span: None,
        }
    }
    
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
    
    pub fn is_unique(&self) -> bool {
        self.attributes.iter().any(|a| matches!(a, Attribute::Unique))
    }
    
    pub fn is_nullable(&self) -> bool {
        self.attributes.iter().any(|a| matches!(a, Attribute::Nullable))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    String,
    Int,
    Bool,
    Double,
    Float,
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
